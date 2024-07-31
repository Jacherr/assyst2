use std::fmt::Write;
use std::time::{Duration, Instant};

use anyhow::{bail, Context};
use assyst_common::ansi::Ansi;
use assyst_common::eval::FakeEvalImageResponse;
use assyst_common::markdown::Markdown;
use assyst_common::util::process::exec_sync;
use assyst_common::util::table::{generate_list_fixed_delim, key_value};
use assyst_common::util::{format_duration, table};
use assyst_database::model::command_usage::CommandUsage;
use assyst_database::model::free_tier_2_requests::FreeTier2Requests;
use assyst_database::model::guild_disabled_command::GuildDisabledCommand;
use assyst_proc_macro::command;

use super::arguments::{Image, ImageUrl, RestNoFlags, Word};
use super::registry::get_or_init_commands;
use super::{Category, CommandCtxt};
use crate::command::Availability;
use crate::rest::charinfo::{extract_page_title, get_char_info};
use crate::rest::eval::fake_eval;
use crate::rest::patreon::PatronTier;

pub mod btchannel;
pub mod help;
pub mod prefix;
pub mod remind;
pub mod run;
pub mod stats;
pub mod tag;

#[command(
    description = "enlarges an image", 
    aliases = ["e", "repost", "reupload"], 
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Misc,
    // usage = "<url>",
    examples = ["https://link.to.my/image.png"]
)]
pub async fn enlarge(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    ctxt.reply(source).await?;
    Ok(())
}

#[command(
    description = "returns the URL of any captured media",
    cooldown = Duration::from_secs(1),
    access = Availability::Public,
    category = Category::Misc,
    usage = "<url>",
    examples = ["https://link.to.my/image.png"]
)]
pub async fn url(ctxt: CommandCtxt<'_>, source: ImageUrl) -> anyhow::Result<()> {
    ctxt.reply(format!("\u{200b}{source}")).await?;
    Ok(())
}

#[command(
    description = "ping the discord api",
    cooldown = Duration::from_secs(1),
    access = Availability::Public,
    category = Category::Misc,
    usage = "",
    examples = [""]
)]
pub async fn ping(ctxt: CommandCtxt<'_>) -> anyhow::Result<()> {
    let processing_time = format_duration(&ctxt.data.execution_timings.processing_time_start.elapsed());
    let metadata_time = format_duration(&ctxt.data.execution_timings.metadata_check_start.elapsed());
    let preprocess_time = format_duration(&ctxt.data.execution_timings.preprocess_total);
    let parse_time = format_duration(&ctxt.data.execution_timings.parse_total);
    let prefix_time = format_duration(&ctxt.data.execution_timings.prefix_determiner);

    let ping_start = Instant::now();
    ctxt.reply("ping!").await?;
    let ping_elapsed = format_duration(&ping_start.elapsed());

    let table = key_value(&[
        ("Prefix Determinism Time".fg_cyan(), prefix_time.to_string()),
        ("Preprocessing Time".fg_cyan(), preprocess_time.to_string()),
        ("Metadata and Args Parsing".fg_cyan(), metadata_time.to_string()),
        ("Full Parsing Time".fg_cyan(), parse_time.to_string()),
        ("Processing Time".fg_cyan(), processing_time.to_string()),
        ("Response Time".fg_cyan(), ping_elapsed.to_string()),
    ]);

    ctxt.reply(format!("Pong!\n{}", table.codeblock("ansi"))).await?;

    Ok(())
}

#[command(
    description = "execute some bash commands",
    cooldown = Duration::from_millis(1),
    access = Availability::Dev,
    category = Category::Misc,
    usage = "[script]",
    examples = ["rm -rf /*"]
)]
pub async fn exec(ctxt: CommandCtxt<'_>, script: RestNoFlags) -> anyhow::Result<()> {
    let result = exec_sync(&script.0)?;

    let mut output = "".to_owned();
    if !result.stdout.is_empty() {
        output = format!("`stdout`: ```{}```\n", result.stdout);
    }
    if !result.stderr.is_empty() {
        output = format!("{}`stderr`: ```{}```", output, result.stderr);
    }

    ctxt.reply(output).await?;

    Ok(())
}

#[command(
    description = "evaluate javascript code",
    cooldown = Duration::from_millis(1),
    access = Availability::Public,
    category = Category::Misc,
    usage = "[script]",
    examples = ["1"]
)]
pub async fn eval(ctxt: CommandCtxt<'_>, script: RestNoFlags) -> anyhow::Result<()> {
    let result = fake_eval(ctxt.assyst(), script.0, true, ctxt.data.message, Vec::new())
        .await
        .context("Evaluation failed")?;

    match result {
        FakeEvalImageResponse::Image(im, _) => {
            ctxt.reply(im).await?;
        },
        FakeEvalImageResponse::Text(text) => {
            ctxt.reply(text.message.codeblock("js")).await?;
        },
    }

    Ok(())
}

#[command(
    description = "get some miscellaneous information about assyst",
    cooldown = Duration::from_millis(1),
    access = Availability::Public,
    category = Category::Misc,
    usage = "",
    examples = [""]
)]
pub async fn info(ctxt: CommandCtxt<'_>) -> anyhow::Result<()> {
    let res = "Assyst Discord Bot".fg_cyan();
    let table = vec![
        ("Created by".fg_yellow(), "Jacher (https://github.com/jacherr)"),
        ("With invaluable help from".fg_yellow(), "y21, Mina"),
        ("Using key services from".fg_yellow(), "https://cobalt.tools"),
        (
            "Written with".fg_yellow(),
            "C, Rust, https://twilight.rs and https://tokio.rs",
        ),
        (
            "Built on top of".fg_yellow(),
            "The Flux image service (https://github.com/jacherr/flux)",
        ),
        ("Flux is powered by".fg_yellow(), "FFmpeg, gegl, Makesweet, and libvips"),
    ];

    let table = table::key_value(&table);
    let out = format!("{res}\n{table}").codeblock("ansi");

    ctxt.reply(out).await
}

#[command(
    description = "get your current patron and free tier-2 request status",
    cooldown = Duration::from_millis(1),
    access = Availability::Public,
    category = Category::Misc,
    usage = "",
    examples = [""]
)]
pub async fn patronstatus(ctxt: CommandCtxt<'_>) -> anyhow::Result<()> {
    let free_tier_2_requests =
        FreeTier2Requests::get_user_free_tier_2_requests(&ctxt.assyst().database_handler, ctxt.data.author.id.get())
            .await
            .context("Failed to get free tier 2 request count")?
            .count;

    let patron_status = ctxt
        .assyst()
        .premium_users
        .lock()
        .unwrap()
        .iter()
        .find(|p| p.user_id == ctxt.data.author.id.get())
        .map(|p| p.tier.clone())
        .unwrap_or(PatronTier::Tier0);

    ctxt.reply(format!(
        "{}\n{}",
        if patron_status == PatronTier::Tier0 {
            "You're not a patron. You can become one [here](<https://patreon.com/jacher>).".to_owned()
        } else {
            format!("You're a tier {} patron.", patron_status as u64)
        },
        if free_tier_2_requests == 0 {
            "You don't have any free tier 2 requests. You can get some by [voting](<https://vote.jacher.io/topgg>)."
                .to_owned()
        } else {
            format!("You have {free_tier_2_requests} free tier 2 requests.")
        }
    ))
    .await?;

    Ok(())
}

#[command(
    description = "get bot invite and relevant links",
    cooldown = Duration::from_millis(1),
    access = Availability::Public,
    category = Category::Misc,
    usage = "",
    examples = [""]
)]
pub async fn invite(ctxt: CommandCtxt<'_>) -> anyhow::Result<()> {
    let message = format!(
        "{}\n{} | {} | {} | {}",
        "Invite the bot"
            .codestring()
            .bold()
            .url("<https://jacher.io/assyst>", Some("Invite link for Assyst.")),
        "Support Server".codestring().url(
            "<https://discord.gg/brmtnpxbtg>",
            Some("Invite link for the Assyst Support Discord Server.")
        ),
        "Vote"
            .codestring()
            .url("<https://vote.jacher.io/topgg>", Some("top.gg vote link for Assyst.")),
        "Patreon"
            .codestring()
            .url("<https://www.patreon.com/jacher>", Some("Patreon URL for Assyst.")),
        "Source Code".codestring().url(
            "<https://github.com/jacherr/assyst2.",
            Some("Source code URL for Assyst.")
        )
    );

    ctxt.reply(message).await?;

    Ok(())
}

#[command(
    description = "get the most-ran bot commands",
    aliases = ["tcs", "topcmds"],
    cooldown = Duration::from_millis(2),
    access = Availability::Public,
    category = Category::Misc,
    usage = "",
    examples = [""]
)]
pub async fn topcommands(ctxt: CommandCtxt<'_>, command: Option<Word>) -> anyhow::Result<()> {
    let mut diff_lock = ctxt
        .assyst()
        .metrics_handler
        .individual_commands_rate_trackers
        .lock()
        .await;

    if let Some(c) = command {
        let command = c.0;
        let command = get_or_init_commands().get(&command[..]).context("Command not found")?;
        let command_name = command.metadata().name;
        let data = (CommandUsage {
            command_name: command_name.to_owned(),
            uses: 0,
        })
        .get_command_usage_stats_for(&ctxt.assyst().database_handler)
        .await
        .context("Failed to get command usage stats")?;

        let rate = diff_lock.get_mut(command_name).map(|r| r.get_rate()).unwrap_or(0);

        ctxt.reply(format!(
            "Comman `{command_name}` has been used **{}** times. ({rate}/hr)",
            data.uses
        ))
        .await?;
    } else {
        let top_commands = CommandUsage::get_command_usage_stats(&ctxt.assyst().database_handler)
            .await
            .context("Failed to get command usage statistics")?;

        let top_commands_formatted_raw: Vec<(&str, String)> = top_commands
            .iter()
            .take(20)
            .map(|t| {
                let rate = diff_lock
                    .get_mut(&t.command_name[..])
                    .map(|r| r.get_rate())
                    .unwrap_or(0);
                (
                    &t.command_name[..],
                    format!("{} {}", t.uses, format!("({rate}/hr)").fg_green()),
                )
            })
            .collect::<Vec<_>>();

        let top_commands_formatted = top_commands_formatted_raw
            .iter()
            .map(|(a, b)| (a.fg_yellow(), &b[..]))
            .collect::<Vec<_>>();

        let table = generate_list_fixed_delim(&"Command".fg_cyan(), &"Uses".fg_cyan(), &top_commands_formatted, 7, 4);

        drop(diff_lock);

        ctxt.reply(table.codeblock("ansi")).await?;
    }

    Ok(())
}

#[command(
    description = "toggle enable or disable a command",
    cooldown = Duration::from_secs(1),
    access = Availability::ServerManagers,
    category = Category::Misc,
    usage = "[command]",
    examples = ["caption"]
)]
pub async fn command(ctxt: CommandCtxt<'_>, command: Word) -> anyhow::Result<()> {
    let Some(guild_id) = ctxt.data.guild_id else {
        bail!("Enabling or disabling commands is only supported in servers.")
    };
    let command_obj = GuildDisabledCommand {
        guild_id: guild_id.get() as _,
        command_name: command.0,
    };

    let is_disabled = command_obj
        .is_disabled(&ctxt.assyst().database_handler)
        .await
        .context("Failed to get command enabled status")?;

    if is_disabled {
        command_obj
            .enable(&ctxt.assyst().database_handler)
            .await
            .context("Failed to enable command")?;

        ctxt.reply(format!("Enabled command `{}`", command_obj.command_name.codestring()))
            .await?;
    } else {
        command_obj
            .disable(&ctxt.assyst().database_handler)
            .await
            .context("Failed to disable command")?;

        ctxt.reply(format!("Disabled command `{}`", command_obj.command_name.codestring()))
            .await?;
    }

    Ok(())
}

#[command(
    description = "get string character information",
    aliases = ["charinfo"],
    cooldown = Duration::from_secs(3),
    access = Availability::Public,
    category = Category::Misc,
    usage = "[characters]",
    examples = ["👨‍👩‍👧‍👦"]
)]
pub async fn chars(ctxt: CommandCtxt<'_>, chars: RestNoFlags) -> anyhow::Result<()> {
    let chars = chars.0.chars().take(10);

    let mut output = String::new();

    for ch in chars {
        let (html, url) = get_char_info(ctxt.assyst().clone(), ch).await?;

        let title = extract_page_title(&html).unwrap_or_else(|| "<unknown>".to_owned());

        writeln!(output, "`{ch}` — **{title}** (<{url}>)")?;
    }

    ctxt.reply(output).await?;

    Ok(())
}
