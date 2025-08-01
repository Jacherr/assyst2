use std::collections::{HashMap, HashSet};
use std::time::Duration;

use anyhow::bail;
use assyst_common::config::CONFIG;
use assyst_proc_macro::command;
use assyst_string_fmt::{Ansi, Markdown};

use crate::assyst::ThreadSafeAssyst;
use crate::command::arguments::WordAutocomplete;
use crate::command::autocomplete::AutocompleteData;
use crate::command::registry::{find_command_by_name, get_or_init_commands};
use crate::command::{Availability, Category, Command, CommandCtxt};

async fn help_autocomplete(_assyst: ThreadSafeAssyst, data: AutocompleteData) -> Vec<String> {
    let commands = get_or_init_commands();
    // remove all aliases
    let mut deduped = HashMap::new();
    for command in commands {
        if command.1.metadata().access != Availability::Dev || CONFIG.dev.admin_users.contains(&data.user.id.get()) {
            deduped.insert(command.1.metadata().name, *command.1);
        }
    }

    let mut names = HashSet::<String>::new();
    for command in deduped {
        if let Some(s) = command.1.subcommands() {
            for c in s {
                names.insert(format!("{} {}", command.0, c.0));
            }
        }
        names.insert(command.0.to_owned());
    }

    names.into_iter().collect()
}

#[command(
    description = "get command help",
    aliases = ["cmds", "commands", "h"],
    cooldown = Duration::from_secs(1),
    access = Availability::Public,
    category = Category::Misc,
    usage = "<category|command>",
    examples = ["", "misc", "ping", "tag create"]
)]
pub async fn help(
    ctxt: CommandCtxt<'_>,
    #[autocomplete = "crate::command::misc::help::help_autocomplete"] labels: Option<Vec<WordAutocomplete>>,
) -> anyhow::Result<()> {
    let cmds = get_or_init_commands();
    let labels = labels.unwrap_or_default();

    // group commands by their category
    let mut groups: HashMap<Category, Vec<_>> = HashMap::new();
    for data in cmds.values() {
        let c = &data.metadata().category;
        groups.entry(c.clone()).or_default();
        let entry = groups.get_mut(&data.metadata().category);

        if let Some(l) = entry
            && !l
                .iter()
                .any(|x: &&&(dyn Command + Send + Sync)| x.metadata().name == data.metadata().name)
        {
            l.push(data);
        }
    }

    let mut labels = labels.into_iter();
    // if we have some argument
    if let Some(WordAutocomplete(base_command)) = labels.next() {
        // if the base is a command
        if let Some(mut command) = find_command_by_name(&base_command) {
            let mut meta = command.metadata();

            let mut usage = format!("{}{}", "Usage: ".fg_yellow(), ctxt.data.calling_prefix);
            let mut name_fmt = meta.name.to_owned();

            // For better error reporting, store the "chain of commands" (e.g. `-t create`)
            let mut command_chain = command.metadata().name.to_owned();

            // If there are more arguments, follow the chain of subcommands and build up the usage
            // along the way
            for WordAutocomplete(mut label) in labels {
                let metadata = command.metadata();
                usage += metadata.name;
                usage += " ";

                label.make_ascii_lowercase();

                let subcommands = command.subcommands();

                match subcommands.and_then(|x| {
                    x.iter()
                        .find(|y| y.0 == label || y.1.metadata().aliases.contains(&label.as_str()))
                        .map(|z| z.1)
                }) {
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

                name_fmt += " ";
                name_fmt += command.metadata().name;
            }

            meta = command.metadata();

            usage += meta.name;
            usage += " ";
            usage += &meta.usage;

            let flags_format = if !meta.flag_descriptions.is_empty() {
                format!(
                    "\n{}",
                    meta.flag_descriptions
                        .iter()
                        .map(|(x, y)| { format!("--{x}: {y}") })
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            } else {
                "None".to_owned()
            };
            let flags = "Flags: ".fg_cyan() + &flags_format;

            let examples_format = if !meta.examples.is_empty() {
                format!(
                    "\n{}",
                    meta.examples
                        .iter()
                        .map(|x| { format!("{}{} {}", ctxt.data.calling_prefix, name_fmt, x) })
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            } else {
                "None".to_owned()
            };
            let examples = "Examples: ".fg_cyan() + &examples_format;

            name_fmt = (name_fmt.clone() + ":").fg_green();
            let description = meta.description;
            let aliases = "Aliases: ".fg_yellow()
                + &(if !meta.aliases.is_empty() {
                    meta.aliases.join(", ")
                } else {
                    "[none]".to_owned()
                });
            let cooldown = format!("{} {} seconds", "Cooldown:".fg_yellow(), meta.cooldown.as_secs());
            let access = "Access: ".fg_yellow() + &meta.access.to_string();
            let subcommands = if let Some(subcommands) = command.subcommands() {
                format!(
                    "\n{}\n{}",
                    "Subcommands:".fg_yellow(),
                    subcommands
                        .iter()
                        .map(|x| format!("\t{} {}", (x.0.to_owned() + ":").fg_red(), x.1.metadata().description))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            } else {
                String::new()
            };

            ctxt.reply(format!("{name_fmt} {description}\n\n{aliases}\n{cooldown}\n{access}\n{usage}{subcommands}\n\n{examples}\n\n{flags}").trim().codeblock("ansi"))
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
                txt += &format!("[{group}]:").fg_green();
                let l = groups.get(&group);

                if let Some(mut list) = l.cloned() {
                    list.sort_by(|a, b| a.metadata().name.cmp(b.metadata().name));

                    for i in list {
                        let name = (i.metadata().name.to_owned() + ":").fg_yellow();
                        txt += &format!("\n\t{name} {}", i.metadata().description);
                    }
                } else {
                    txt += &"\n\t[no commands]".fg_black();
                }

                ctxt.reply(txt.codeblock("ansi")).await?;
            }
        }
    } else {
        let mut msg = String::new();
        let mut sorted = groups.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| format!("{}", a.0).cmp(&format!("{}", b.0)));

        for (group, list) in sorted {
            let mut commands = list.iter().map(|x| x.metadata().name).collect::<Vec<_>>();
            commands.sort_unstable();

            msg += &format!(
                "{}{} {}\n\n",
                "[".fg_yellow() + &group.fg_yellow() + &"]".fg_yellow(),
                ':'.fg_yellow(),
                commands.join(", ")
            );
        }

        msg = msg.trim().codeblock("ansi");

        msg += &format!(
            "\nUse {} for more information on a command, or {} for more information on a category.\n\n",
            format!("{}help [command]", ctxt.data.calling_prefix).codestring(),
            format!("{}help [category]", ctxt.data.calling_prefix).codestring()
        );

        msg += &format!(
            "{} | {} | {} | {} | {} | {}",
            "Invite"
                .codestring()
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
            "Server Premium".codestring().url(
                format!(
                    " <https://discord.com/application-directory/571661221854707713/store/{}>",
                    CONFIG.entitlements.premium_server_sku_id
                ),
                Some("Link to the SKU for Assyst premium servers.")
            ),
            "Source Code".codestring().url(
                "<https://github.com/jacherr/assyst2>",
                Some("Source code URL for Assyst.")
            )
        );

        ctxt.reply(msg).await?;
    }

    Ok(())
}
