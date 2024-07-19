use std::collections::HashMap;
use std::sync::OnceLock;

use tracing::info;
use twilight_model::application::command::Command as InteractionCommand;

use crate::assyst::ThreadSafeAssyst;
use crate::command::CommandMetadata;

use super::{fun, image, misc, services, TCommand};

macro_rules! declare_commands {
    ($($name:path),*) => {
        const RAW_COMMANDS: &[TCommand] = &[
            $(&$name as TCommand),*
        ];
    }
}

declare_commands!(
    fun::colour::colour_command,
    fun::findsong_command,
    fun::translation::bad_translate_command,
    fun::translation::translate_command,
    image::ahshit_command,
    image::aprilfools_command,
    image::audio::drip_command,
    image::audio::femurbreaker_command,
    image::audio::siren_command,
    image::audio::sweden_command,
    image::audio::terraria_command,
    image::bloom_command,
    image::blur_command,
    image::caption_command,
    image::deepfry_command,
    image::fisheye_command,
    image::flip_command,
    image::flop_command,
    image::frames_command,
    image::frameshift_command,
    image::ghost_command,
    image::gif_command,
    image::gifmagik_command,
    image::globe_command,
    image::grayscale_command,
    image::image_info_command,
    image::invert_command,
    image::jpeg_command,
    image::magik_command,
    image::makesweet::back_tattoo_command,
    image::makesweet::billboard_command,
    image::makesweet::book_command,
    image::makesweet::circuitboard_command,
    image::makesweet::flag_command,
    image::makesweet::flag2_command,
    image::makesweet::fortunecookie_command,
    image::makesweet::heartlocket_command,
    image::makesweet::rubiks_command,
    image::makesweet::toaster_command,
    image::makesweet::valentine_command,
    image::meme_command,
    image::motivate_command,
    image::neon_command,
    image::pingpong_command,
    image::resize_command,
    image::reverse_command,
    image::scramble_command,
    image::speed_command,
    misc::enlarge_command,
    misc::eval_command,
    misc::exec_command,
    misc::help::help_command,
    misc::info_command,
    misc::ping_command,
    misc::remind::remind_command,
    misc::run::run_command,
    misc::stats::stats_command,
    misc::tag::tag_command,
    misc::url_command,
    services::burntext_command,
    services::cooltext::cooltext_command,
    services::download_command,
    services::r34_command
);

static COMMANDS: OnceLock<HashMap<&'static str, TCommand>> = OnceLock::new();

/// Prefer [`find_command_by_name`] where possible.
pub fn get_or_init_commands() -> &'static HashMap<&'static str, TCommand> {
    COMMANDS.get_or_init(|| {
        let mut map = HashMap::new();

        for &command in RAW_COMMANDS {
            let &CommandMetadata { name, aliases, .. } = command.metadata();
            map.insert(name, command);
            info!("Registering command {} (aliases={:?})", name, aliases);
            for alias in aliases {
                map.insert(alias, command);
            }
        }

        map
    })
}

/// Finds a command by its name.
pub fn find_command_by_name(name: &str) -> Option<TCommand> {
    get_or_init_commands().get(name).copied()
}

pub async fn register_interaction_commands(assyst: ThreadSafeAssyst) -> anyhow::Result<Vec<InteractionCommand>> {
    let commands = get_or_init_commands()
        .iter()
        .map(|x| x.1.as_interaction_command())
        .filter(|x| !x.name.is_empty())
        .collect::<Vec<_>>();

    // deduplicate out aliases
    let mut deduplicated_commands: Vec<InteractionCommand> = vec![];
    for command in commands {
        if !deduplicated_commands.iter().any(|x| x.name == command.name) {
            deduplicated_commands.push(command);
        }
    }

    let response = assyst
        .interaction_client()
        .set_global_commands(&deduplicated_commands)
        .await?
        .model()
        .await?;

    Ok(response)
}
