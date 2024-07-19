use std::time::Duration;

use assyst_common::ansi::Ansi;
use assyst_common::markdown::Markdown;
use assyst_common::util::process::{
    exec_sync, get_processes_cpu_usage, get_processes_mem_usage, get_processes_uptimes,
};
use assyst_proc_macro::command;
use human_bytes::human_bytes;
use twilight_model::gateway::SessionStartLimit;

use crate::command::arguments::Word;
use crate::command::misc::key_value;
use crate::command::{Availability, Category, CommandCtxt};
use crate::rest::filer::{get_filer_stats as filer_stats, FilerStats};

#[command(
    description = "get bot stats",
    cooldown = Duration::from_secs(5),
    access = Availability::Public,
    category = Category::Misc,
    usage = "<section>",
    examples = ["", "sessions", "storage", "all"],
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

        let table = key_value(&[
            ("Total".fg_cyan(), total.to_string()),
            ("Remaining".fg_cyan(), remaining.to_string()),
            ("Max Concurrency".fg_cyan(), max_concurrency.to_string()),
        ]);

        Ok(table.codeblock("ansi"))
    }

    async fn get_storage_stats(ctxt: &CommandCtxt<'_>) -> anyhow::Result<String> {
        let database_size = ctxt.assyst().database_handler.database_size().await?.size;
        let cache_size = human_bytes(ctxt.assyst().database_handler.cache.size_of() as f64);
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

    async fn get_general_stats(ctxt: &CommandCtxt<'_>) -> String {
        let events_rate = ctxt.assyst().metrics_handler.get_events_rate();
        let events_total = ctxt.assyst().metrics_handler.events.get();
        let commands_rate = ctxt.assyst().metrics_handler.get_commands_rate().to_string();
        let commit = exec_sync("git rev-parse HEAD")
            .map(|x| x.stdout[..8].to_owned())
            .unwrap_or("Unknown".to_string());

        let stats_table = key_value(&[
            (
                "Guilds".fg_cyan(),
                ctxt.assyst().metrics_handler.guilds.get().to_string(),
            ),
            ("Shards".fg_cyan(), ctxt.assyst().shard_count.to_string()),
            (
                "Events".fg_cyan(),
                format!("{events_rate}/sec ({events_total} since restart)"),
            ),
            ("Commands Executed".fg_cyan(), commands_rate + "/min"),
            ("Commit Hash".fg_cyan(), commit),
            (
                "Flux Version".fg_cyan(),
                ctxt.assyst()
                    .flux_handler
                    .get_version()
                    .await
                    .unwrap_or("Unknown".to_owned()),
            ),
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
        let stats_table = get_general_stats(&ctxt).await;
        let usages_table = get_process_stats();
        let storage_table = get_storage_stats(&ctxt).await?;
        let session_table = get_session_stats(&ctxt).await?;

        let full_output = format!(
            "**General**\n{stats_table}\n**Processes**\n{usages_table}\n**Storage and Caches**\n{storage_table}\n**Sessions**\n{session_table}"
        );

        ctxt.reply(full_output).await?;
    } else {
        // default to general and process stats
        let stats_table = get_general_stats(&ctxt).await;
        let usages_table = get_process_stats();

        let msg = format!("{} {}", stats_table, usages_table);

        ctxt.reply(msg).await?;
    }

    Ok(())
}
