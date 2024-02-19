use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::command::Availability;

use super::arguments::{self, Image, ImageUrl, Rest, Time, Word};
use super::registry::get_or_init_commands;
use super::{Category, Command, CommandCtxt};

use assyst_common::ansi::Ansi;
use assyst_common::markdown::Markdown;
use assyst_proc_macro::command;

#[command(
    name = "remind",
    aliases = ["reminder"],
    description = "get reminders or set a reminder, time format is xdyhzm (check examples)",
    access = Availability::Public,
    cooldown = Duration::from_secs(2),
    category = Category::Misc,
    examples = [],
)]
pub async fn remind(_ctxt: CommandCtxt<'_>, _when: Time, _text: Rest) -> anyhow::Result<()> {
    Ok(())
}

#[command(description = "enlarges an image", cooldown = Duration::ZERO, access = Availability::Public, category = Category::Misc)]
pub async fn e(ctxt: CommandCtxt<'_>, source: Image) -> anyhow::Result<()> {
    ctxt.reply(source).await?;
    Ok(())
}

#[command(description = "returns the URL of any captured media", cooldown = Duration::ZERO, access = Availability::Public, category = Category::Misc)]
pub async fn url(ctxt: CommandCtxt<'_>, source: ImageUrl) -> anyhow::Result<()> {
    ctxt.reply(format!("\u{200b}{source}")).await?;
    Ok(())
}

#[command(description = "ping the discord api", cooldown = Duration::ZERO, access = Availability::Public, category = Category::Misc)]
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

#[command(description = "get command help", cooldown = Duration::ZERO, access = Availability::Public, category = Category::Misc)]
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
            ctxt.reply(
                format!(
                    "{}{}{}\n{}\n\n{}\n{} {} {}\n{} {}\n",
                    Ansi::underline(&meta.name.fg_yellow()),
                    ":".fg_yellow(),
                    "\x1b[0m", // dude fuck discord
                    meta.description,
                    if !meta.aliases.is_empty() {
                        format!("{} {}", " Aliases:".fg_yellow(), meta.aliases.join(", "))
                    } else {
                        "[none]".fg_black()
                    },
                    "Cooldown:".fg_yellow(),
                    meta.cooldown.as_secs(),
                    "seconds",
                    "  Access:".fg_yellow(),
                    meta.access
                )
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
                txt += &Ansi::underline(&format!("{g}:").fg_yellow());
                txt += "\x1b[0m"; // again
                let l = groups.get(&g);

                if let Some(list) = l {
                    for i in list {
                        txt += &format!("\n\t{}: {}", i.metadata().name, i.metadata().description.fg_black())
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
