use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::command::Availability;
use crate::rest::filer::{get_filer_stats as filer_stats, FilerStats};

use super::arguments::{Image, ImageUrl, Rest, Time, Word};
use super::registry::{find_command_by_name, get_or_init_commands};
use super::{Category, Command, CommandCtxt};

use anyhow::bail;
use assyst_common::ansi::Ansi;
use assyst_common::markdown::Markdown;
use assyst_common::util::format_duration;
use assyst_common::util::process::{get_processes_cpu_usage, get_processes_mem_usage, get_processes_uptimes};
use assyst_common::util::table::key_value;
use assyst_proc_macro::command;
use human_bytes::human_bytes;
use twilight_model::gateway::SessionStartLimit;

pub mod tag;

#[command(
    name = "remind",
    aliases = ["reminder"],
    description = "get reminders or set a reminder, time format is xdyhzm (check examples)",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Misc,
    usage = "[time] <message>",
    examples = ["2h do the laundry", "3d30m hand assignment in", "30m"],
)]
pub async fn remind(_ctxt: CommandCtxt<'_>, _when: Time, _text: Rest) -> anyhow::Result<()> {
    Ok(())
}

#[command(
    description = "enlarges an image", 
    aliases = ["e"], 
    cooldown = Duration::from_secs(2),
    access = Availability::Public,
    category = Category::Misc,
    usage = "<url>",
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

    let table = key_value(&vec![
        ("Full Processing Time".fg_cyan(), format!("{processing_time}")),
        ("Metadata and Args Parsing".fg_cyan(), format!("{metadata_time}")),
        ("Preprocessing Time".fg_cyan(), format!("{preprocess_time}")),
        ("Prefix Determinism Time".fg_cyan(), format!("{prefix_time}")),
        ("Full Parsing Time".fg_cyan(), format!("{parse_time}")),
        ("Response Time".fg_cyan(), format!("{ping_elapsed}")),
    ]);

    ctxt.reply(format!("Pong!\n{}", table.codeblock("ansi"))).await?;

    Ok(())
}

#[command(
    description = "get command help",
    cooldown = Duration::from_secs(1),
    access = Availability::Public,
    category = Category::Misc,
    usage = "<category|command>",
    examples = ["", "misc", "ping", "tag create"]
)]
pub async fn help(ctxt: CommandCtxt<'_>, labels: Vec<Word>) -> anyhow::Result<()> {
    let cmds = get_or_init_commands();

    // group commands by their category
    let mut groups: HashMap<Category, Vec<_>> = HashMap::new();
    for data in cmds.values() {
        let c = &data.metadata().category;
        groups.entry(c.clone()).or_default();
        let entry = groups.get_mut(&data.metadata().category);

        if let Some(l) = entry {
            if !l
                .iter()
                .any(|x: &&&(dyn Command + Send + Sync)| x.metadata().name == data.metadata().name)
            {
                l.push(data);
            }
        }
    }

    let mut labels = labels.into_iter();
    // if we have some argument
    if let Some(Word(base_command)) = labels.next() {
        // if the base is a command
        if let Some(mut command) = find_command_by_name(&base_command) {
            let mut usage = format!("{}{}", "Usage: ".fg_yellow(), ctxt.data.calling_prefix);

            // For better error reporting, store the "chain of commands" (e.g. `-t create`)
            let mut command_chain = command.metadata().name.to_owned();

            // If there are more arguments, follow the chain of subcommands and build up the usage along the way
            for Word(mut label) in labels.into_iter() {
                let metadata = command.metadata();
                usage += metadata.name;
                usage += " ";

                label.make_ascii_lowercase();

                match command.subcommand(&label) {
                    Some(sc) => command = sc,
                    None => bail!(
                        "subcommand {} does not exist (use {}help {})",
                        label,
                        ctxt.data.calling_prefix,
                        command_chain
                    ),
                }

                command_chain += " ";
                command_chain += command.metadata().name;
            }
            usage += command.metadata().name;
            usage += " ";
            usage += command.metadata().usage;

            let meta = command.metadata();

            let name_fmt = (meta.name.to_owned() + ":").fg_green();
            let description = meta.description;
            let aliases = "Aliases: ".fg_yellow()
                + &(if !meta.aliases.is_empty() {
                    meta.aliases.join(",")
                } else {
                    "[none]".to_owned()
                });
            let cooldown = format!("{} {} seconds", "Cooldown:".fg_yellow(), meta.cooldown.as_secs());
            let access = "Access: ".fg_yellow() + &meta.access.to_string();

            let examples_format = if !meta.examples.is_empty() {
                format!(
                    "\n{}",
                    meta.examples
                        .iter()
                        .map(|x| { format!("{}{} {}", ctxt.data.calling_prefix, meta.name, x) })
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            } else {
                "None".to_owned()
            };
            let examples = "Examples: ".fg_cyan() + &examples_format;

            ctxt.reply(
                format!("{name_fmt} {description}\n\n{aliases}\n{cooldown}\n{access}\n{usage}\n\n{examples}")
                    .trim()
                    .codeblock("ansi"),
            )
            .await?;
        } else {
            // ... if it isn't a command, then go check if it's a category
            let group: Category = base_command.clone().into();

            // if its a category
            if let Category::None(_) = group {
                ctxt.reply(format!(
                    "{} No command or group named {} found.",
                    emoji::symbols::warning::WARNING.glyph,
                    base_command.codestring()
                ))
                .await?;
            // irrelevant
            } else {
                let mut txt = String::new();
                txt += &format!("{group}:").fg_green();
                let l = groups.get(&group);

                if let Some(list) = l {
                    for i in list {
                        let name = (i.metadata().name.to_owned() + ":").fg_yellow();
                        txt += &format!("\n\t{name} {}", i.metadata().description)
                    }
                } else {
                    txt += &"\n\t[no commands]".fg_black()
                }

                ctxt.reply(txt.codeblock("ansi")).await?;
            }
        }
    } else {
        let mut msg = String::new();
        for (group, list) in groups {
            msg += &format!(
                "{}{} {}\n",
                group.fg_yellow(),
                ':'.fg_yellow(),
                list.iter().map(|x| x.metadata().name).collect::<Vec<_>>().join(", ")
            );
        }

        msg = msg.codeblock("ansi");

        msg += &format!(
            "\nUse {} for more information on a command.\n\n",
            "-help [command]".codestring()
        );

        msg += &format!(
            "{} | {} | {}",
            "Invite".url("<https://jacher.io/assyst>", Some("Invite link for Assyst.")),
            "Support Server".url(
                "<https://discord.gg/brmtnpxbtg>",
                Some("Invite link for the Assyst Support Discord Server.")
            ),
            "Vote".url("<https://vote.jacher.io/topgg>", Some("top.gg vote link for Assyst."))
        );

        ctxt.reply(msg).await?;
    }

    Ok(())
}

#[command(
    description = "get bot stats",
    cooldown = Duration::from_secs(5),
    access = Availability::Public,
    category = Category::Misc,
    usage = "<section>",
    examples = ["", "sessions", "storage", "all"],
    aliases = ["info"],
    send_processing = true
)]
pub async fn stats(ctxt: CommandCtxt<'_>, option: Option<Word>) -> anyhow::Result<()> {
    fn get_process_stats() -> String {
        let mem_usages = get_processes_mem_usage();
        let mem_usages_fmt = mem_usages
            .iter()
            .map(|x| (x.0.fg_cyan(), human_bytes(x.1 as f64)))
            .collect::<Vec<_>>();

        let cpu_usages = get_processes_cpu_usage();
        let cpu_usages_fmt = cpu_usages
            .iter()
            .map(|x| (x.0.fg_cyan(), format!("{:.2?}", x.1) + "%"))
            .collect::<Vec<_>>();

        let uptimes = get_processes_uptimes();
        let uptimes_fmt = uptimes.iter().map(|(x, y)| (x.fg_cyan(), y)).collect::<Vec<_>>();

        let mut combined_usages: Vec<(String, String)> = vec![];
        for i in mem_usages_fmt {
            if let Some(cpu) = cpu_usages_fmt.iter().find(|x| x.0 == i.0)
                && let Some(uptime) = uptimes_fmt.iter().find(|x| x.0 == i.0)
            {
                combined_usages.push((
                    cpu.0.clone(),
                    format!("Memory: {}, CPU: {}, Uptime: {}", i.1, cpu.1, uptime.1),
                ));
            }
        }

        let usages_table = key_value(&combined_usages);

        usages_table.codeblock("ansi")
    }

    async fn get_session_stats(ctxt: &CommandCtxt<'_>) -> anyhow::Result<String> {
        let gateway_bot = ctxt.assyst().http_client.gateway().authed().await?.model().await?;
        let SessionStartLimit {
            total,
            remaining,
            max_concurrency,
            ..
        } = gateway_bot.session_start_limit;

        let table = key_value(&vec![
            ("Total".fg_cyan(), total.to_string()),
            ("Remaining".fg_cyan(), remaining.to_string()),
            ("Max Concurrency".fg_cyan(), max_concurrency.to_string()),
        ]);

        Ok(table.codeblock("ansi"))
    }

    async fn get_storage_stats(ctxt: &CommandCtxt<'_>) -> anyhow::Result<String> {
        let database_size = ctxt.assyst().database_handler.read().await.database_size().await?.size;
        let cache_size = human_bytes(ctxt.assyst().database_handler.read().await.cache.size_of() as f64);
        let filer_stats = filer_stats(ctxt.assyst().clone()).await.unwrap_or(FilerStats {
            count: 0,
            size_bytes: 0,
        });
        let rest_cache_size = human_bytes(ctxt.assyst().rest_cache_handler.size_of() as f64);

        let table = key_value(&vec![
            ("Database Total Size".fg_cyan(), database_size),
            ("Database Cache Total Size".fg_cyan(), cache_size),
            ("Filer File Count".fg_cyan(), filer_stats.count.to_string()),
            ("Filer Total Size".fg_cyan(), human_bytes(filer_stats.size_bytes as f64)),
            ("Rest Cache Total Size".fg_cyan(), rest_cache_size),
        ]);

        Ok(table.codeblock("ansi"))
    }

    fn get_general_stats(ctxt: &CommandCtxt<'_>) -> String {
        let events_rate = ctxt
            .assyst()
            .prometheus
            .get_events_rate()
            .map(|x| x.to_string())
            .unwrap_or("0".to_owned());
        let commands_rate = ctxt
            .assyst()
            .prometheus
            .get_commands_rate()
            .map(|x| x.to_string())
            .unwrap_or("0".to_owned());

        let stats_table = key_value(&vec![
            ("Guilds".fg_cyan(), ctxt.assyst().prometheus.guilds.get().to_string()),
            ("Shards".fg_cyan(), ctxt.assyst().shard_count.to_string()),
            ("Events".fg_cyan(), events_rate + "/sec"),
            ("Commands".fg_cyan(), commands_rate + "/min"),
        ]);

        stats_table.codeblock("ansi")
    }

    if let Some(Word(ref x)) = option
        && x.to_lowercase() == "sessions"
    {
        let table = get_session_stats(&ctxt).await?;

        ctxt.reply(table).await?;
    } else if let Some(Word(ref x)) = option
        && (x.to_lowercase() == "caches" || x.to_lowercase() == "storage")
    {
        let table = get_storage_stats(&ctxt).await?;

        ctxt.reply(table).await?;
    } else if let Some(Word(ref x)) = option
        && x.to_lowercase() == "all"
    {
        let stats_table = get_general_stats(&ctxt);
        let usages_table = get_process_stats();
        let storage_table = get_storage_stats(&ctxt).await?;
        let session_table = get_session_stats(&ctxt).await?;

        let full_output = format!(
            "**General**\n{stats_table}\n**Processes**\n{usages_table}\n**Storage and Caches**\n{storage_table}\n**Sessions**\n{session_table}"
        );

        ctxt.reply(full_output).await?;
    } else {
        // default to general and process stats
        let stats_table = get_general_stats(&ctxt);
        let usages_table = get_process_stats();

        let msg = format!("{} {}", stats_table, usages_table);

        ctxt.reply(msg).await?;
    }

    Ok(())
}
