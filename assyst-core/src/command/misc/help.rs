use std::collections::HashMap;
use std::time::Duration;

use anyhow::bail;
use assyst_common::ansi::Ansi;
use assyst_common::markdown::Markdown;
use assyst_proc_macro::command;

use crate::command::arguments::Word;
use crate::command::registry::{find_command_by_name, get_or_init_commands};
use crate::command::{Availability, Category, Command, CommandCtxt};

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
