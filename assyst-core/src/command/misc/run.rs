use std::time::{Duration, Instant};

use anyhow::Context;
use assyst_common::markdown::Markdown;
use assyst_common::util::process::{exec_sync, CommandOutput};
use assyst_proc_macro::command;
use dash_rt::format_value;
use dash_vm::eval::EvalError;
use dash_vm::value::Root;
use dash_vm::Vm;

use crate::command::arguments::Codeblock;
use crate::command::flags::{ChargeFlags, RustFlags};
use crate::command::messagebuilder::{Attachment, MessageBuilder};
use crate::command::{Availability, Category, CommandCtxt};
use crate::define_commandgroup;
use crate::rest::rust::{run_benchmark, run_binary, run_clippy, run_godbolt, run_miri, OptimizationLevel};

struct ExecutableDeletionDefer(String);
impl Drop for ExecutableDeletionDefer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0.clone());
    }
}

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
        flags_string += " --print-llir-only --no-timings"
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
                format!("{} (execution time {:?})", b.exit_code.to_string(), bin_time)
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
            })
            .await?;
        }
    }

    Ok(())
}

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
                        format!("Exception: {}", f.to_string())
                    } else {
                        format!("Exception: {:?}", fmt.unwrap_err())
                    }
                },
                EvalError::Middle(middle) => format!("Middle error: {:?}", middle),
            },
        }
    };

    ctxt.reply(str_result).await?;
    Ok(())
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
        "dash" => dash
    ]
}
