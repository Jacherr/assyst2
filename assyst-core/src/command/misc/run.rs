use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::{bail, Context};
use assyst_common::util::process::{exec_sync, exec_sync_in_dir, CommandOutput};
use assyst_proc_macro::command;
use assyst_string_fmt::Markdown;
use dash_rt::format_value;
use dash_vm::eval::EvalError;
use dash_vm::value::Root;
use dash_vm::Vm;
use serde::Deserialize;
use tokio::fs;
use toml::from_str;

use crate::command::arguments::Codeblock;
use crate::command::flags::{flags_from_str, FlagDecode, FlagType};
use crate::command::messagebuilder::{Attachment, MessageBuilder};
use crate::command::{Availability, Category, CommandCtxt};
use crate::downloader::download_content;
use crate::rest::rust::{run_benchmark, run_binary, run_clippy, run_godbolt, run_miri, OptimizationLevel};
use crate::{define_commandgroup, flag_parse_argument};

struct ExecutableDeletionDefer(String);
impl Drop for ExecutableDeletionDefer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0.clone());
    }
}

#[derive(Default)]
pub struct ChargeFlags {
    pub verbose: bool,
    pub llir: bool,
    pub opt: u64,
    pub valgrind: bool,
}
impl FlagDecode for ChargeFlags {
    fn from_str(input: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut valid_flags = HashMap::new();
        valid_flags.insert("verbose", FlagType::NoValue);
        valid_flags.insert("llir", FlagType::NoValue);
        valid_flags.insert("opt", FlagType::WithValue);
        valid_flags.insert("valgrind", FlagType::NoValue);

        let raw_decode = flags_from_str(input, valid_flags)?;
        let opt = raw_decode
            .get("opt")
            .and_then(|x| x.as_deref())
            .map(|x| x.parse::<u64>())
            .unwrap_or(Ok(0))
            .context("Failed to parse optimisation level")?;

        let result = Self {
            verbose: raw_decode.contains_key("verbose"),
            llir: raw_decode.contains_key("llir"),
            opt,
            valgrind: raw_decode.contains_key("valgrind"),
        };

        if result.llir && result.valgrind {
            bail!("Cannot set both valgrind and llir flags at the same time");
        }

        Ok(result)
    }
}
flag_parse_argument! { ChargeFlags }

#[command(
    description = "execute some charge",
    cooldown = Duration::from_millis(100),
    access = Availability::Dev,
    category = Category::Misc,
    usage = "[script] <flags>",
    examples = ["fn main(): i32 { return 1; }"],
    send_processing = true,
    flag_descriptions = [
        ("verbose", "Get verbose output"),
        ("llir", "Output LLVM IR"),
        ("opt [level:0|1|2|3]", "Set optimisation level of LLVM"),
        ("valgrind", "Run output executable in valgrind")
    ]
)]
pub async fn charge(ctxt: CommandCtxt<'_>, script: Codeblock, flags: ChargeFlags) -> anyhow::Result<()> {
    let dir = "/tmp/charge".to_owned();
    if std::fs::metadata(format!("{dir}/.git")).is_err() {
        let _ = std::fs::remove_dir_all(&dir);
        exec_sync(&format!("git clone https://github.com/y21/lang.git {dir} --depth=1"))?;
    };

    exec_sync(&format!("cd {dir} && git pull"))?;
    std::fs::write(format!("{dir}/input"), script.0).context("Failed to write input file")?;
    exec_sync(&format!("cd {dir} && npm i --save-dev @types/node && tsc"))?;

    let commit_hash = exec_sync(&format!("cd {dir} && git rev-parse HEAD"))
        .map(|x| x.stdout[..8].to_owned())
        .unwrap_or("Unknown".to_owned());

    let mut flags_string = String::new();
    if flags.verbose {
        flags_string += "--verbose"
    };

    if flags.llir {
        flags_string += " --print-llir-only"
    }

    flags_string += &format!(" -O{}", flags.opt);

    let result = exec_sync(&format!("cd {dir} && node . input {}", flags_string.trim()))?;

    if !flags.llir {
        let executable = format!("{dir}/a.out");

        #[allow(unused)]
        let exec_defer = ExecutableDeletionDefer(executable.clone());

        let bin_start = Instant::now();
        let bin_result = if std::fs::metadata(executable).is_ok() {
            Some(exec_sync(&format!(
                "cd {dir} && {}./a.out",
                if flags.valgrind { "valgrind -q " } else { "" }
            ))?)
        } else {
            None
        };
        let bin_time = bin_start.elapsed();

        let stdout = result.stdout + "\n" + &bin_result.clone().unwrap_or(CommandOutput::default()).stdout;
        let stderr = result.stderr + "\n" + &bin_result.clone().unwrap_or(CommandOutput::default()).stderr;

        let mut output = "".to_owned();
        if !stdout.trim().is_empty() {
            output = format!("`stdout`: {}\n", stdout.codeblock("ansi"));
        }

        if !stderr.trim().is_empty() {
            output = format!("{}`stderr`: {}\n", output, stderr.codeblock("ansi"));
        }

        output.push_str(&format!(
            "\nCompiler: {}\nExecutable: {}\nCommit Hash: {commit_hash}",
            result.exit_code,
            if let Some(b) = bin_result {
                format!("{} (execution time {:?})", b.exit_code, bin_time)
            } else {
                "N/A".to_owned()
            }
        ));

        ctxt.reply(output).await?;
    } else {
        let stdout = result.stdout;
        let stderr = result.stderr;

        if result.exit_code.code() != Some(0) {
            ctxt.reply(format!("Compilation failed: {}", stderr.codeblock("")))
                .await?;
        } else if stdout.split("\n").count() < 100 {
            ctxt.reply(stdout.codeblock("llvm").to_string()).await?;
        } else {
            ctxt.reply(MessageBuilder {
                content: None,
                attachment: Some(Attachment {
                    name: "out.txt".into(),
                    data: stdout.as_bytes().to_vec(),
                }),
                components: None,
                component_ctxt: None,
            })
            .await?;
        }
    }

    Ok(())
}

#[derive(Default)]
pub struct RustFlags {
    pub miri: bool,
    pub asm: bool,
    pub clippy: bool,
    pub bench: bool,
    pub release: bool,
}
impl FlagDecode for RustFlags {
    fn from_str(input: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut valid_flags = HashMap::new();
        valid_flags.insert("miri", FlagType::NoValue);
        valid_flags.insert("release", FlagType::NoValue);
        valid_flags.insert("asm", FlagType::NoValue);
        valid_flags.insert("clippy", FlagType::NoValue);
        valid_flags.insert("bench", FlagType::NoValue);

        let raw_decode = flags_from_str(input, valid_flags)?;
        let result = Self {
            miri: raw_decode.contains_key("miri"),
            asm: raw_decode.contains_key("asm"),
            release: raw_decode.contains_key("release"),
            clippy: raw_decode.contains_key("clippy"),
            bench: raw_decode.contains_key("bench"),
        };

        Ok(result)
    }
}
flag_parse_argument! { RustFlags }

#[command(
    description = "execute some rust",
    cooldown = Duration::from_millis(100),
    access = Availability::Public,
    category = Category::Misc,
    usage = "[script] <flags>",
    examples = ["println!(\"Hello, world!\")"],
    send_processing = true,
    flag_descriptions = [
        ("miri", "Run code in miri debugger"),
        ("asm", "Output ASM of Rust code"),
        ("clippy", "Lint code using Clippy"),
        ("bench", "Run code as a benchmark"),
        ("release", "Run code in release mode")
    ]
)]
pub async fn rust(ctxt: CommandCtxt<'_>, script: Codeblock, flags: RustFlags) -> anyhow::Result<()> {
    let opt = if flags.release {
        OptimizationLevel::Release
    } else {
        OptimizationLevel::Debug
    };

    let result = if flags.miri {
        run_miri(&ctxt.assyst().reqwest_client, &script.0, "nightly", opt).await?
    } else if flags.asm {
        run_godbolt(&ctxt.assyst().reqwest_client, &script.0).await?
    } else if flags.clippy {
        run_clippy(&ctxt.assyst().reqwest_client, &script.0, "nightly", opt).await?
    } else if flags.bench {
        run_benchmark(&ctxt.assyst().reqwest_client, &script.0).await?
    } else {
        run_binary(&ctxt.assyst().reqwest_client, &script.0, "nightly", opt).await?
    };

    ctxt.reply(result.format().codeblock("rs")).await
}

#[command(
    description = "execute some dash",
    cooldown = Duration::from_millis(100),
    access = Availability::Dev,
    category = Category::Misc,
    usage = "[script]",
    examples = ["\"Hello, world!\""],
    send_processing = true
)]
pub async fn dash(ctxt: CommandCtxt<'_>, script: Codeblock) -> anyhow::Result<()> {
    let str_result = {
        let mut vm = Vm::new(Default::default());
        let result = vm.eval(&script.0, Default::default());
        let mut scope = vm.scope();
        match result {
            Ok(result) => {
                let fmt = format_value(result.root(&mut scope), &mut scope);
                if let Ok(f) = fmt {
                    f.to_string()
                } else {
                    format!("{:?}", fmt.unwrap_err())
                }
            },
            Err(err) => match err {
                EvalError::Exception(unrooted) => {
                    let fmt = format_value(unrooted.root(&mut scope), &mut scope);
                    if let Ok(f) = fmt {
                        format!("Exception: {f}")
                    } else {
                        format!("Exception: {:?}", fmt.unwrap_err())
                    }
                },
                EvalError::Middle(middle) => format!("Middle error: {middle:?}"),
            },
        }
    };

    ctxt.reply(str_result).await?;
    Ok(())
}

const RUSTC_BOILERPLATE: &str = r##"#![feature(rustc_private)]

extern crate rustc_ast_pretty;
extern crate rustc_driver;
extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_middle;

use std::{path, process, str, sync::Arc};

use rustc_errors::registry;
use rustc_session::config;

fn run<F>(script: &'static str, f: F) where F: FnOnce(rustc_middle::ty::TyCtxt) + Send + Sync {
let out = process::Command::new("rustc")
        .arg("--print=sysroot")
        .current_dir(".")
        .output()
        .unwrap();
    let sysroot = str::from_utf8(&out.stdout).unwrap().trim();
    let config = rustc_interface::Config {
        opts: config::Options {
            maybe_sysroot: Some(path::PathBuf::from(sysroot)),
            ..config::Options::default()
        },
        input: config::Input::Str {
            name: rustc_span::FileName::Custom("main.rs".to_string()),
            input: script.to_string(),
        },
        crate_cfg: Vec::new(),
        crate_check_cfg: Vec::new(),
        output_dir: None,
        output_file: None,
        file_loader: None,
        locale_resources: rustc_driver::DEFAULT_LOCALE_RESOURCES,
        lint_caps: rustc_hash::FxHashMap::default(),
        psess_created: None,
        register_lints: None,
        override_queries: None,
        make_codegen_backend: None,
        registry: registry::Registry::new(rustc_errors::codes::DIAGNOSTICS),
        expanded_args: Vec::new(),
        ice_file: None,
        hash_untracked_state: None,
        using_internal_features: Arc::default(),
    };
    rustc_interface::run_compiler(config, |compiler| {
        compiler.enter(|queries| {
            // Analyze the crate and inspect the types under the cursor.
            queries.global_ctxt().unwrap().enter(f)
        });
    });
}

fn main() {
    {{code}}
}"##;

#[command(
    description = "execute some rustc",
    cooldown = Duration::from_millis(100),
    access = Availability::Dev,
    category = Category::Misc,
    usage = "[script]",
    examples = ["run(\"fn main() {}\", |tcx| { dbg!(tcx.hir().root_module()); });"],
    send_processing = true,
)]
pub async fn rustc(ctxt: CommandCtxt<'_>, script: Codeblock) -> anyhow::Result<()> {
    let script = RUSTC_BOILERPLATE.replace("{code}", &script.0);
    let project_dir = "/tmp/_assyst_rustc_dev";

    if fs::metadata(project_dir).await.is_err() {
        fs::create_dir(project_dir)
            .await
            .context("Failed to create project directory")?;

        exec_sync(&format!("cd {project_dir} && cargo init")).context("Failed to initialise rustc project")?;

        // create copy of old
        fs::copy(
            format!("{project_dir}/Cargo.toml"),
            format!("{project_dir}/_Cargo.toml"),
        )
        .await
        .context("Failed to copy Cargo.toml")?;
    }

    // replace old config with fresh
    fs::copy(
        format!("{project_dir}/_Cargo.toml"),
        format!("{project_dir}/Cargo.toml"),
    )
    .await
    .context("Failed to copy _Cargo.toml")?;

    // refresh dependencies each time in case they change
    let deps = vec!["clippy_utils = { git = \"https://github.com/rust-lang/rust-clippy\" }"];
    let mut cargo = fs::read_to_string(format!("{project_dir}/Cargo.toml"))
        .await
        .context("Failed to read Cargo.toml")?;

    for dep in deps {
        cargo += "\n";
        cargo += dep;
    }

    fs::write(format!("{project_dir}/Cargo.toml"), cargo)
        .await
        .context("Failed to write dependencies to Cargo.toml")?;

    let script_path = format!("{project_dir}/src/main.rs");
    fs::write(script_path, script)
        .await
        .context("Failed to write main.rs")?;

    // download toolchain so we use correct compiler for clippy_utils and other internal crates
    let raw = download_content(
        &ctxt.assyst().reqwest_client,
        "https://raw.githubusercontent.com/rust-lang/rust-clippy/master/rust-toolchain",
        usize::MAX,
        false,
    )
    .await
    .context("Failed to download rust-toolchain")?;

    let toolchain = String::from_utf8_lossy(&raw);
    #[derive(Deserialize)]
    struct Toolchain {
        pub toolchain: Channel,
    }
    #[derive(Deserialize)]
    struct Channel {
        pub channel: String,
    }

    fs::write(format!("{project_dir}/rust-toolchain"), toolchain.to_string())
        .await
        .context("Failed to write rust-toolchain")?;

    let channel = from_str::<Toolchain>(&toolchain)
        .context("Failed to deserialize toolchain")?
        .toolchain
        .channel;

    let result = exec_sync_in_dir(
        &format!("rustup install {channel} && rustup component add rust-src rustc-dev llvm-tools-preview"),
        project_dir,
    )?;

    if !result.exit_code.success() {
        ctxt.reply(format!(
            "Failed to install components: {}",
            result.stderr.codeblock("rs")
        ))
        .await?;
        return Ok(());
    }

    ctxt.reply("Compiling and executing...").await?;

    let result = exec_sync_in_dir(&format!("mold -run cargo +{channel} run -q"), project_dir)
        .context("Failed to execute binary")?;

    if !result.exit_code.success() {
        ctxt.reply(format!("Failed to execute script: {}", result.stderr.codeblock("rs")))
            .await?;
        return Ok(());
    }

    let mut output = "".to_owned();
    if !result.stdout.is_empty() {
        output = format!("`stdout`: ```{}```\n", result.stdout);
    }
    if !result.stderr.is_empty() {
        output = format!("{}`stderr`: ```{}```", output, result.stderr);
    }

    ctxt.reply(output).await
}

define_commandgroup! {
    name: run,
    access: Availability::Public,
    category: Category::Misc,
    description: "run code via various runtimes and languages",
    usage: "[language/runtime] [code] <...flags>",
    commands: [
        "charge" => charge,
        "rust" => rust,
        "dash" => dash,
        "rustc" => rustc
    ]
}
