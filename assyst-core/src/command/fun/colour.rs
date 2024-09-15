use std::collections::HashMap;
use std::time::Duration;

use anyhow::{bail, Context};
use assyst_common::err;
use assyst_database::model::colour_role::ColourRole;
use assyst_proc_macro::command;
use assyst_string_fmt::Markdown;
use twilight_model::id::marker::{GuildMarker, RoleMarker};
use twilight_model::id::Id;

use crate::assyst::ThreadSafeAssyst;
use crate::command::arguments::{Word, WordAutocomplete};
use crate::command::autocomplete::AutocompleteData;
use crate::command::flags::{flags_from_str, FlagDecode, FlagType};
use crate::command::{Availability, Category, CommandCtxt};
use crate::{define_commandgroup, flag_parse_argument};

const DEFAULT_COLOURS: &[(&str, u32)] = &[
    ("gold", 0xf1c40f),
    ("teal", 0x1abc9c),
    ("darkpurple", 0x71368a),
    ("darkblue", 0x206694),
    ("salmon", 0xffa07a),
    ("lavender", 0xd1d1ff),
    ("lightred", 0xff4c4c),
    ("yellow", 0xfbf606),
    ("pink", 0xff69b4),
    ("lime", 0xff00),
    ("cyan", 0x8f8fc),
    ("white", 0xffffff),
    ("black", 0x10101),
    ("orange", 0xe67e22),
    ("blue", 0x3498db),
    ("purple", 0x8b00ff),
    ("green", 0x2ecc71),
    ("red", 0xe74c3c),
];

pub async fn colour_role_autocomplete(assyst: ThreadSafeAssyst, autocomplete_data: AutocompleteData) -> Vec<String> {
    let roles = match ColourRole::list_in_guild(
        &assyst.database_handler,
        autocomplete_data.guild_id.unwrap().get() as i64,
    )
    .await
    {
        Ok(l) => l,
        Err(e) => {
            err!("Error fetching colour roles for autocompletion: {e:?}");
            vec![]
        },
    };

    roles.iter().map(|x| &x.name).cloned().collect::<Vec<_>>()
}

#[command(
    aliases = [],
    description = "Add a new colour role",
    access = Availability::ServerManagers,
    cooldown = Duration::from_secs(5),
    category = Category::Fun,
    usage = "[name] [colour code]",
    examples = ["red #ff0000"],
)]
pub async fn add(ctxt: CommandCtxt<'_>, name: Word, code: Word) -> anyhow::Result<()> {
    if let Some(id) = ctxt.data.guild_id.map(|x| x.get()) {
        if name.0.contains(" ") {
            bail!("Colour role names cannot contain spaces");
        };

        if name.0 == "add"
            || name.0 == "remove"
            || name.0 == "add-defaults"
            || name.0 == "remove-all"
            || name.0 == "reset"
        {
            bail!("Colour role name cannot be a reserved word");
        }

        let name = name.0.to_ascii_lowercase();

        let roles = ColourRole::list_in_guild(&ctxt.assyst().database_handler, id as i64)
            .await
            .context("Failed to fetch existing colour roles")?;

        if roles.iter().any(|x| x.name == name) {
            bail!("A colour role with the name `{}` already exists in this server.", name);
        }

        let colour =
            u32::from_str_radix(code.0.strip_prefix("#").unwrap_or(&code.0), 16).context("Invalid colour code")?;

        let role = ctxt
            .assyst()
            .http_client
            .create_role(Id::<GuildMarker>::new(id))
            .name(&name)
            .color(colour)
            .await
            .context("Failed to create colour role")?
            .model()
            .await
            .context("Failed to create colour role")?;

        let colour_entry = ColourRole {
            name,
            role_id: role.id.get() as i64,
            guild_id: id as i64,
        };

        colour_entry
            .insert(&ctxt.assyst().database_handler)
            .await
            .context("Failed to register colour role in database")?;

        ctxt.reply(format!("Successfully registered colour role `{}`", colour_entry.name))
            .await?;

        Ok(())
    } else {
        bail!("This command is only supported inside Discord servers.");
    }
}

#[command(
    name = "add-default",
    aliases = [],
    description = "Add default colour roles",
    access = Availability::ServerManagers,
    cooldown = Duration::from_secs(20),
    category = Category::Fun,
    usage = "",
    examples = [""],
)]
pub async fn add_default(ctxt: CommandCtxt<'_>) -> anyhow::Result<()> {
    if let Some(id) = ctxt.data.guild_id.map(|x| x.get()) {
        let current = ColourRole::list_in_guild(&ctxt.assyst().database_handler, id as i64)
            .await
            .context("Failed to fetch existing colour roles")?;

        let mut count_created = 0;

        for entry in DEFAULT_COLOURS {
            if !current.iter().any(|x| x.name == entry.0) {
                let role = ctxt
                    .assyst()
                    .http_client
                    .create_role(Id::<GuildMarker>::new(id))
                    .name(entry.0)
                    .color(entry.1)
                    .await
                    .context(format!("Failed to create colour role {}", entry.0))?
                    .model()
                    .await
                    .context(format!("Failed to create colour role {}", entry.0))?;

                let colour_entry = ColourRole {
                    name: entry.0.to_owned(),
                    role_id: role.id.get() as i64,
                    guild_id: id as i64,
                };

                colour_entry
                    .insert(&ctxt.assyst().database_handler)
                    .await
                    .context(format!("Failed to register colour role {}", entry.0))?;

                count_created += 1;
            }
        }

        ctxt.reply(format!("{count_created} default colour roles have been created."))
            .await?;

        Ok(())
    } else {
        bail!("This command is only supported inside Discord servers.");
    }
}

#[command(
    aliases = [],
    description = "Remove an existing colour role from the server",
    access = Availability::ServerManagers,
    cooldown = Duration::from_secs(5),
    category = Category::Fun,
    usage = "[name]",
    examples = ["red"],
)]
pub async fn remove(
    ctxt: CommandCtxt<'_>,
    #[autocomplete = "crate::command::fun::colour::colour_role_autocomplete"] name: WordAutocomplete,
) -> anyhow::Result<()> {
    if let Some(id) = ctxt.data.guild_id.map(|x| x.get()) {
        let colour = name.0.to_ascii_lowercase();

        let roles = ColourRole::list_in_guild(&ctxt.assyst().database_handler, id as i64)
            .await
            .context("Failed to fetch existing colour roles")?;

        let role = match roles.iter().find(|x| x.name == colour) {
            Some(role) => role,
            None => bail!(
                "Colour role {colour} does not exist in this server (use {}colour for a list).",
                ctxt.data.calling_prefix
            ),
        };

        role.remove(&ctxt.assyst().database_handler)
            .await
            .context("Failed to unregister colour role")?;

        ctxt.assyst()
            .http_client
            .delete_role(Id::<GuildMarker>::new(id), Id::<RoleMarker>::new(role.role_id as u64))
            .await
            .context("Failed to delete role from Discord")?;

        ctxt.reply(format!("Successfully deleted colour role {}", role.name))
            .await?;
    } else {
        bail!("This command is only supported inside Discord servers.");
    }

    Ok(())
}

#[command(
    name = "remove-all",
    aliases = [],
    description = "Remove all existing colour roles from the server (THIS CANNOT BE UNDONE)",
    access = Availability::ServerManagers,
    cooldown = Duration::from_secs(20),
    category = Category::Fun,
    usage = "--i-am-sure",
    examples = ["", "--i-am-sure"],
    flag_descriptions = [
        ("i-am-sure", "Confirm this operation"),
    ]
)]
pub async fn remove_all(ctxt: CommandCtxt<'_>, flags: ColourRemoveAllFlags) -> anyhow::Result<()> {
    if let Some(id) = ctxt.data.guild_id.map(|x| x.get()) {
        if flags.i_am_sure {
            let roles = ColourRole::list_in_guild(&ctxt.assyst().database_handler, id as i64)
                .await
                .context("Failed to fetch existing colour roles")?;

            for role in roles {
                role.remove(&ctxt.assyst().database_handler)
                    .await
                    .context(format!("Failed to unregister colour role {}", role.name))?;

                ctxt.assyst()
                    .http_client
                    .delete_role(Id::<GuildMarker>::new(id), Id::<RoleMarker>::new(role.role_id as u64))
                    .await
                    .context(format!("Failed to delete colour role {} from Discord", role.name))?;
            }

            ctxt.reply("All colour roles in this server have been PERMANENTLY DELETED.")
                .await?;
        } else {
            bail!(
                "Please confirm your intention by rerunning this command with the --i-am-sure flag: \"{}colour remove-all --i-am-sure\"",
                ctxt.data.calling_prefix
            );
        }
    } else {
        bail!("This command is only supported inside Discord servers.");
    }

    Ok(())
}

#[command(
    aliases = [],
    description = "Remove all colour roles from yourself",
    access = Availability::ServerManagers,
    cooldown = Duration::from_secs(20),
    category = Category::Fun,
    usage = "",
    examples = [""],
)]
pub async fn reset(ctxt: CommandCtxt<'_>) -> anyhow::Result<()> {
    if let Some(id) = ctxt.data.guild_id.map(|x| x.get()) {
        let colour_roles = ColourRole::list_in_guild(&ctxt.assyst().database_handler, id as i64)
            .await
            .context("Failed to fetch existing colour roles")?;

        let user_roles = ctxt
            .assyst()
            .http_client
            .guild_member(Id::<GuildMarker>::new(id), ctxt.data.author.id)
            .await
            .context("Failed to fetch user")?
            .model()
            .await
            .context("Failed to fetch user")?
            .roles;

        let user_roles_minus_colours = user_roles
            .iter()
            .filter(|r| colour_roles.iter().all(|x| x.role_id as u64 != r.get()))
            .copied()
            .collect::<Vec<_>>();

        ctxt.assyst()
            .http_client
            .update_guild_member(Id::<GuildMarker>::new(id), ctxt.data.author.id)
            .roles(&user_roles_minus_colours)
            .await
            .context("Failed to update user roles")?;

        ctxt.reply("Your colour roles have been removed.").await?;
    } else {
        bail!("This command is only supported inside Discord servers.");
    };

    Ok(())
}

#[command(
    aliases = [],
    description = "Assign yourself a colour role or list all colour roles",
    access = Availability::Public,
    cooldown = Duration::from_secs(5),
    category = Category::Services,
    usage = "",
    examples = [""],
)]
pub async fn default(
    ctxt: CommandCtxt<'_>,
    #[autocomplete = "crate::command::fun::colour::colour_role_autocomplete"] colour: Option<WordAutocomplete>,
) -> anyhow::Result<()> {
    if let Some(id) = ctxt.data.guild_id.map(|x| x.get()) {
        if let Some(colour) = colour.map(|x| x.0.to_ascii_lowercase()) {
            let roles = ColourRole::list_in_guild(&ctxt.assyst().database_handler, id as i64)
                .await
                .context("Failed to get colour roles")?;

            let role = match roles.iter().find(|x| x.name == colour) {
                Some(role) => role,
                None => bail!(
                    "Colour role {colour} does not exist in this server (use {}colour for a list).",
                    ctxt.data.calling_prefix
                ),
            };

            let user_id = ctxt.data.author.id;

            let user_roles = ctxt
                .assyst()
                .http_client
                .guild_member(Id::<GuildMarker>::new(id), user_id)
                .await?
                .model()
                .await?
                .roles;

            let mut user_roles_minus_colours = user_roles
                .iter()
                .filter(|r| roles.iter().all(|x| x.role_id as u64 != r.get()))
                .copied()
                .collect::<Vec<_>>();

            user_roles_minus_colours.push(Id::<RoleMarker>::new(role.role_id as u64));

            ctxt.assyst()
                .http_client
                .update_guild_member(Id::<GuildMarker>::new(id), user_id)
                .roles(&user_roles_minus_colours)
                .await?;

            ctxt.reply(format!("Your colour role is now `{colour}`.")).await?;
        } else {
            let roles = ColourRole::list_in_guild(&ctxt.assyst().database_handler, id as i64)
                .await
                .context("Failed to get colour roles")?;

            let formatted = roles.iter().map(|r| r.name.clone()).collect::<Vec<String>>().join(", ");
            let reply = format!(
                "Available colours:\n{}\nUse `{}colour [colour name]` to assign a colour to yourself.",
                formatted.codeblock(""),
                ctxt.data.calling_prefix
            );

            ctxt.reply(reply).await?;
        }
    } else {
        bail!("This command is only supported inside Discord servers.");
    }

    Ok(())
}

define_commandgroup! {
    name: colour,
    access: Availability::Public,
    category: Category::Fun,
    aliases: ["color", "colours", "colors"],
    cooldown: Duration::from_secs(5),
    description: "Assyst colour roles",
    examples: ["red", "", "add red #ff0000", "add-default", "remove red", "reset", "remove-all"],
    usage: "[colour]",
    guild_only: true,
    commands: [
        "add" => add,
        "add-default" => add_default,
        "remove" => remove,
        "remove-all" => remove_all,
        "reset" => reset
    ],
    default_interaction_subcommand: "assign",
    default: default
}

#[derive(Default)]
pub struct ColourRemoveAllFlags {
    pub i_am_sure: bool,
}
impl FlagDecode for ColourRemoveAllFlags {
    fn from_str(input: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut valid_flags = HashMap::new();
        valid_flags.insert("i-am-sure", FlagType::NoValue);

        let raw_decode = flags_from_str(input, valid_flags)?;
        let result = Self {
            i_am_sure: raw_decode.contains_key("i-am-sure"),
        };

        Ok(result)
    }
}
flag_parse_argument! { ColourRemoveAllFlags }
