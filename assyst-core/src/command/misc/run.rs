use std::time::{Duration, SystemTime, UNIX_EPOCH};

use assyst_common::markdown::Markdown;
use assyst_common::util::process::{exec_sync, CommandOutput};
use assyst_proc_macro::command;

use crate::command::arguments::Codeblock;
use crate::command::flags::{LangFlags, RustFlags};
use crate::command::{Availability, Category, CommandCtxt};
use crate::define_commandgroup;
use crate::rest::rust::{run_benchmark, run_binary, run_clippy, run_godbolt, run_miri, OptimizationLevel};

#[command(
    description = "execute some lang",
    cooldown = Duration::from_millis(100),
    access = Availability::Dev,
    category = Category::Misc,
    usage = "[script] <flags: --verbose>",
    examples = ["1"],
    send_processing = true
)]
pub async fn lang(ctxt: CommandCtxt<'_>, script: Codeblock, flags: LangFlags) -> anyhow::Result<()> {
    let dir = format!(
        "/tmp/lang/{}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
    );

    exec_sync(&format!("git clone https://github.com/y21/lang.git {dir}"))?;
    std::fs::write(format!("{dir}/input"), script.0)?;

    exec_sync(&format!("cd {dir} && npm i --save-dev @types/node && tsc"))?;
    let result = exec_sync(&format!(
        "cd {dir} && node . input {}",
        if flags.verbose { "--verbose" } else { "" }
    ))?;

    let bin_result = if std::fs::metadata(format!("{dir}/a.out")).is_ok() {
        Some(exec_sync(&format!("cd {dir} && ./a.out"))?)
    } else {
        None
    };

    let stdout = result.stdout + "\n" + &bin_result.clone().unwrap_or(CommandOutput::default()).stdout;
    let stderr = result.stderr + "\n" + &bin_result.clone().unwrap_or(CommandOutput::default()).stderr;

    let mut output = "".to_owned();
    if !stdout.trim().is_empty() {
        output = format!("`stdout`: ```ansi\n{}```\n", stdout);
    }

    if !stderr.trim().is_empty() {
        output = format!("{}`stderr`: ```ansi\n{}```", output, stderr);
    }
    output.push_str(&format!(
        "\nCompiler: {}\nExecutable: {}",
        result.exit_code,
        if let Some(b) = bin_result {
            b.exit_code.to_string()
        } else {
            "N/A".to_owned()
        }
    ));

    // todo: delete `dir`

    ctxt.reply(output).await?;

    Ok(())
}

#[command(
    description = "execute some rust",
    cooldown = Duration::from_millis(100),
    access = Availability::Public,
    category = Category::Misc,
    usage = "[script] <flags: --miri|--asm|--clippy|--bench|--release>",
    examples = ["println!(\"Hello World!\")"],
    send_processing = true
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

define_commandgroup! {
    name: run,
    access: Availability::Public,
    category: Category::Misc,
    description: "run code via various runtimes and languages",
    usage: "[language/runtime] [code] <...flags>",
    commands: [
        "lang" => lang,
        "rust" => rust
    ]
}
