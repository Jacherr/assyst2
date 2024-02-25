use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::command::Availability;
use crate::rest::filer::get_filer_stats as filer_stats;

use super::arguments::{Image, ImageUrl, Rest, Time, Word};
use super::registry::get_or_init_commands;
use super::{Category, Command, CommandCtxt};

use human_bytes::human_bytes;
use assyst_common::ansi::Ansi;
use assyst_common::markdown::Markdown;
use assyst_common::util::process::{get_processes_cpu_usage, get_processes_mem_usage, get_processes_uptimes};
use assyst_common::util::table::key_value;
use assyst_proc_macro::command;
use twilight_model::gateway::SessionStartLimit;

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
    let processing_time = ctxt.data.processing_time_start.elapsed();
    let ping_start = Instant::now();
    ctxt.reply("ping!").await?;
    let ping_elapsed = ping_start.elapsed();
    ctxt.reply(format!(
        "pong!\nprocessing time: {processing_time:?}\nresponse time: {ping_elapsed:?}"
    ))
    .await?;

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
pub async fn help(ctxt: CommandCtxt<'_>, label: Option<Word>) -> anyhow::Result<()> {
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

    // if we have some argument
    if let Some(l) = label {
        let tx = l.0.to_lowercase();

        // if said argument is a command
        if let Some(cmd) = cmds.get(&*tx) {
            let meta = &cmd.metadata();
            let name_fmt = (meta.name.to_owned() + ":").fg_green();
            let description = meta.description;
            let aliases = "Aliases: ".fg_yellow() + &(if !meta.aliases.is_empty() {
                meta.aliases.join(",")
            } else {
                "[none]".to_owned()
            });
            let cooldown = format!("{} {} seconds", "Cooldown:".fg_yellow(), meta.cooldown.as_secs());
            let access = "Access: ".fg_yellow() + &meta.access.to_string();
            let usage = "Usage: ".fg_yellow() + &format!("{}{} {}", ctxt.data.calling_prefix, meta.name, meta.usage.to_string());

            let examples_format = if !meta.examples.is_empty() { 
                format!("\n{}", meta.examples.iter().map(|x| {
                    format!("{}{} {}", ctxt.data.calling_prefix, meta.name, x)
                }).collect::<Vec<_>>().join("\n")) 
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
            // otherwise, its either irrelevant or a category
        } else {
            let g: Category = tx.clone().into();

            // if its a category
            if let Category::None(_) = g {
                ctxt.reply(format!(
                    "{} No command or group named {} found.",
                    emoji::symbols::warning::WARNING.glyph,
                    tx.codestring()
                ))
                .await?;
            // irrelevant
            } else {
                let mut txt = String::new();
                txt += &format!("{g}:").fg_green();
                let l = groups.get(&g);

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
                "{}{} {}",
                group.fg_yellow(),
                ':'.fg_yellow(),
                list.iter().map(|x| x.metadata().name).collect::<Vec<_>>().join(", ")
            );
        }

        msg = msg.codeblock("ansi");

        msg += &format!(
            "\nDo {} for more info on a command.\n\n",
            "-help [command]".codestring()
        );

        msg += &format!(
            "{} | {} | {}",
            "Invite".url("<https://jacher.io/assyst>", Some("Invite link for Assyst.")),
            "Support Server".url(
                "<https://discord.gg/brmtnpxbtg>",
                Some("Invite link for the Assyst Support Discord Server.")
            ),
            "top.gg".url("<https://vote.jacher.io/topgg>", Some("top.gg vote link for Assyst."))
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
    examples = ["", "sessions", "database", "filer"],
    aliases = ["info"]
)]
pub async fn stats(ctxt: CommandCtxt<'_>, option: Option<Word>) -> anyhow::Result<()> {
    fn get_process_stats() -> String {
        let mem_usages = get_processes_mem_usage();
        let mem_usages_fmt = mem_usages.iter()
            .map(|x| (x.0.fg_cyan(), human_bytes(x.1 as f64)))
            .collect::<Vec<_>>();
    
        let cpu_usages = get_processes_cpu_usage();
        let cpu_usages_fmt = cpu_usages.iter()
            .map(|x| (x.0.fg_cyan(), format!("{:.2?}", x.1) + "%"))
            .collect::<Vec<_>>();
    
        let uptimes = get_processes_uptimes();
        let uptimes_fmt = uptimes.iter()
                .map(|(x, y)| (x.fg_cyan(), y))
                .collect::<Vec<_>>();
    
        let mut combined_usages: Vec<(String, String)> = vec![];
        for i in mem_usages_fmt {
            if let Some(cpu) = cpu_usages_fmt.iter().find(|x| x.0 == i.0)
                && let Some(uptime) = uptimes_fmt.iter().find(|x| x.0 == i.0) {
                combined_usages.push(
                    (
                        cpu.0.clone(),
                        format!("Memory: {}, CPU: {}, Uptime: {}", i.1, cpu.1, uptime.1)
                    )
                );
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
            ("Max Concurrency".fg_cyan(), max_concurrency.to_string())
        ]);

        Ok(table.codeblock("ansi"))
    }

    async fn get_database_stats(ctxt: &CommandCtxt<'_>) -> anyhow::Result<String> {
        let database_size = ctxt.assyst().database_handler.read().await.database_size().await?.size;
        let cache_size = human_bytes(ctxt.assyst().database_handler.read().await.cache.size_of() as f64);

        let table = key_value(&vec![
            ("Database Size".fg_cyan(), database_size),
            ("Cache Size".fg_cyan(), cache_size)
        ]);

        Ok(table.codeblock("ansi"))
    }

    async fn get_filer_stats(ctxt: &CommandCtxt<'_>) -> anyhow::Result<String> {
        let stats = filer_stats(ctxt.assyst().clone()).await?;

        let table = key_value(&vec![
            ("File Count".fg_cyan(), stats.count.to_string()),
            ("Total Size".fg_cyan(), human_bytes(stats.size_bytes as f64))
        ]);

        Ok(table.codeblock("ansi"))
    }

    ctxt.reply("Collecting...").await?;

    if let Some(Word(ref x)) = option && x.to_lowercase() == "sessions" {
        let table = get_session_stats(&ctxt).await?;

        ctxt.reply(table).await?;
    } else if let Some(Word(ref x)) = option && (x.to_lowercase() == "database" || x.to_lowercase() == "db") {
        let table = get_database_stats(&ctxt).await?;

        ctxt.reply(table).await?;
    } else if let Some(Word(ref x)) = option && (x.to_lowercase() == "filer" || x.to_lowercase() == "cdn") {
        let table = get_filer_stats(&ctxt).await?;

        ctxt.reply(table).await?;
    } else {
        let events_rate = ctxt.assyst().prometheus.get_events_rate().map(|x| x.to_string()).unwrap_or("0".to_owned());
        let commands_rate = ctxt.assyst().prometheus.get_commands_rate().map(|x| x.to_string()).unwrap_or("0".to_owned());
    
        let stats_table = key_value(&vec![
            ("Guilds".fg_cyan(), ctxt.assyst().prometheus.guilds.get().to_string()),
            ("Shards".fg_cyan(), ctxt.assyst().shard_count.to_string()),
            ("Events".fg_cyan(), events_rate + "/sec"),
            ("Commands".fg_cyan(), commands_rate + "/min")
        ]);
    
        let usages_table = get_process_stats();

        let msg = format!("{} {}", stats_table.codeblock("ansi"), usages_table);
    
        ctxt.reply(msg).await?;
    }

    Ok(())
}