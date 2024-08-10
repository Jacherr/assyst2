use std::fmt::Display;

use assyst_common::util::discord::{channel_mention_to_id, get_avatar_url, id_from_mention, user_mention_to_id};
use assyst_common::util::{parse_to_millis, regex};
use assyst_string_fmt::markdown::parse_codeblock;
use serde::Deserialize;
use twilight_model::application::command::CommandOption;
use twilight_model::application::interaction::application_command::CommandOptionValue;
use twilight_model::channel::message::sticker::{MessageSticker, StickerFormatType};
use twilight_model::channel::message::Embed;
use twilight_model::channel::{Attachment, Channel as TwlChannel};
use twilight_model::id::marker::{ChannelMarker, UserMarker};
use twilight_model::id::Id;
use twilight_model::user::User as TwlUser;
use twilight_util::builder::command::{
    AttachmentBuilder, ChannelBuilder, IntegerBuilder, NumberBuilder, StringBuilder, UserBuilder,
};

use super::errors::{ArgsExhausted, TagParseError};
use super::{CommandCtxt, InteractionCommandParseCtxt, Label, RawMessageParseCtxt};
use crate::assyst::Assyst;
use crate::commit_if_ok;
use crate::downloader::{self, ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES};
use crate::gateway_handler::message_parser::error::{ErrorSeverity, GetErrorSeverity};

pub trait ParseArgument: Sized {
    /// Parses `Self`, given a command, where the source is a raw message.
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError>;
    /// Parses `Self`, given a command, where the source is an interaction command.
    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError>;
    fn as_command_option(name: &str) -> CommandOption;
    fn usage(name: &str) -> String {
        format!("<{name}>")
    }
}

impl ParseArgument for u64 {
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        let word = ctxt.next_word(label)?;
        Ok(word.parse()?)
    }

    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let next = &ctxt.option_by_name(&label.unwrap().0)?.value;
        if let CommandOptionValue::Integer(option) = next {
            Ok(*option as u64)
        } else {
            // cloning is fine since this should (ideally) never happen
            Err(TagParseError::MismatchedCommandOptionType((
                "u64".to_owned(),
                next.clone(),
            )))
        }
    }

    fn as_command_option(name: &str) -> CommandOption {
        IntegerBuilder::new(name, "integer option").required(true).build()
    }
}

impl ParseArgument for f64 {
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        let word = ctxt.next_word(label)?;
        Ok(word.parse()?)
    }

    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let next = &ctxt.option_by_name(&label.unwrap().0)?.value;
        if let CommandOptionValue::Number(option) = next {
            Ok(*option)
        } else {
            Err(TagParseError::MismatchedCommandOptionType((
                "f64".to_owned(),
                next.clone(),
            )))
        }
    }

    fn as_command_option(name: &str) -> CommandOption {
        NumberBuilder::new(name, "number option").required(true).build()
    }
}

impl ParseArgument for f32 {
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        let word = ctxt.next_word(label)?;
        Ok(word.parse()?)
    }

    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let next = &ctxt.option_by_name(&label.unwrap().0)?.value;
        if let CommandOptionValue::Number(option) = next {
            Ok(*option as f32)
        } else {
            Err(TagParseError::MismatchedCommandOptionType((
                "f32".to_owned(),
                next.clone(),
            )))
        }
    }

    fn as_command_option(name: &str) -> CommandOption {
        NumberBuilder::new(name, "number option").required(true).build()
    }
}

impl<T: ParseArgument> ParseArgument for Option<T> {
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        // TODO: should we be using commit_if_ok to undo failed parsers?
        match T::parse_raw_message(ctxt, label).await {
            Ok(v) => Ok(Some(v)),
            Err(err) if err.get_severity() == ErrorSeverity::High => Err(err),
            _ => Ok(None),
        }
    }

    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        // TODO: should we be using commit_if_ok to undo failed parsers?
        match T::parse_command_option(ctxt, label).await {
            Ok(v) => Ok(Some(v)),
            Err(err) if err.get_severity() == ErrorSeverity::High => Err(err),
            _ => Ok(None),
        }
    }

    fn as_command_option(name: &str) -> CommandOption {
        let mut option = T::as_command_option(name);
        option.required = Some(false);
        option
    }

    fn usage(name: &str) -> String {
        let as_required = T::usage(name);
        // this is hacky maybe
        format!("[{}]", &as_required[1..as_required.len() - 1])
    }
}

impl ParseArgument for Vec<Word> {
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        let mut items = Vec::new();

        // `Option<T>`'s parser takes care of recovering from low severity errors
        // and any `Err`s returned are fatal, so we can just use `?`
        while let Some(value) = <Option<Word>>::parse_raw_message(ctxt, label.clone()).await? {
            items.push(value)
        }

        Ok(items)
    }

    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let text = Word::parse_command_option(ctxt, label).await?;
        let items = text
            .0
            .split_ascii_whitespace()
            .map(|y| Word(y.to_owned()))
            .collect::<Vec<_>>();

        Ok(items)
    }

    fn as_command_option(name: &str) -> CommandOption {
        StringBuilder::new(name, "text input").required(true).build()
    }

    fn usage(name: &str) -> String {
        format!("<{name}[]>")
    }
}

/// A time argument such as `1h20m30s`.
#[derive(Debug)]
pub struct Time {
    pub millis: u64,
}
impl ParseArgument for Time {
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        let word = ctxt.next_word(label)?;
        let millis = parse_to_millis(word)?;

        Ok(Time { millis })
    }

    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let word = &ctxt.option_by_name(&label.unwrap().0)?.value;

        if let CommandOptionValue::String(ref option) = word {
            let millis = parse_to_millis(option)?;

            Ok(Time { millis })
        } else {
            Err(TagParseError::MismatchedCommandOptionType((
                "String (time)".to_owned(),
                word.clone(),
            )))
        }
    }

    fn as_command_option(name: &str) -> CommandOption {
        StringBuilder::new(name, "time input").required(true).build()
    }
}

/// A single word argument.
#[derive(Debug)]
pub struct Word(pub String);

impl ParseArgument for Word {
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        Ok(Self(ctxt.next_word(label)?.to_owned()))
    }

    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let word = &ctxt.option_by_name(&label.unwrap().0)?.value;

        if let CommandOptionValue::String(ref option) = word {
            Ok(Word(option.clone()))
        } else {
            Err(TagParseError::MismatchedCommandOptionType((
                "String".to_owned(),
                word.clone(),
            )))
        }
    }

    fn as_command_option(name: &str) -> CommandOption {
        StringBuilder::new(name, "word input").required(true).build()
    }
}

/// A codeblock argument (may also be plaintext).
#[derive(Debug)]
pub struct Codeblock(pub String);
impl ParseArgument for Codeblock {
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        let fork = ctxt.fork();
        let all = fork.rest_all(label.clone());
        if all.ends_with("```") {
            Ok(Codeblock(parse_codeblock(ctxt.rest_all(label))))
        } else {
            Ok(Codeblock(parse_codeblock(ctxt.rest(label)?)))
        }
    }

    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let word = &ctxt.option_by_name(&label.unwrap().0)?.value;

        if let CommandOptionValue::String(ref option) = word {
            Ok(Codeblock(option.clone()))
        } else {
            Err(TagParseError::MismatchedCommandOptionType((
                "String".to_owned(),
                word.clone(),
            )))
        }
    }

    fn as_command_option(name: &str) -> CommandOption {
        StringBuilder::new(name, "code argument").required(true).build()
    }
}

/// A user argument (mention or ID)
#[derive(Debug)]
pub struct User(pub TwlUser);
impl ParseArgument for User {
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        let next = ctxt.next_word(label)?;
        let id = user_mention_to_id(next);

        let user = ctxt
            .cx
            .assyst()
            .http_client
            .user(Id::<UserMarker>::new(id.unwrap_or(next.parse::<u64>().unwrap_or(1))))
            .await
            .map_err(|e| TagParseError::TwilightHttp(Box::new(e)))?
            .model()
            .await
            .map_err(|e| TagParseError::TwilightDeserialize(Box::new(e)))?;

        Ok(User(user))
    }

    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let word = &ctxt.option_by_name(&label.unwrap().0)?.value;

        if let CommandOptionValue::User(id) = word {
            let user = ctxt
                .cx
                .assyst()
                .http_client
                .user(*id)
                .await
                .map_err(|e| TagParseError::TwilightHttp(Box::new(e)))?
                .model()
                .await
                .map_err(|e| TagParseError::TwilightDeserialize(Box::new(e)))?;

            Ok(User(user))
        } else {
            Err(TagParseError::MismatchedCommandOptionType((
                "User".to_owned(),
                word.clone(),
            )))
        }
    }

    fn as_command_option(name: &str) -> CommandOption {
        UserBuilder::new(name, "user argument").required(true).build()
    }
}

/// A user argument (mention or ID)
#[derive(Debug)]
pub struct Channel(pub TwlChannel);
impl ParseArgument for Channel {
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        let next = ctxt.next_word(label)?;
        let id = channel_mention_to_id(next);

        let channel = ctxt
            .cx
            .assyst()
            .http_client
            .channel(Id::<ChannelMarker>::new(id.unwrap_or(next.parse::<u64>().unwrap_or(1))))
            .await
            .map_err(|e| TagParseError::TwilightHttp(Box::new(e)))?
            .model()
            .await
            .map_err(|e| TagParseError::TwilightDeserialize(Box::new(e)))?;

        Ok(Channel(channel))
    }

    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let word = &ctxt.option_by_name(&label.unwrap().0)?.value;

        if let CommandOptionValue::Channel(id) = word {
            let channel = ctxt
                .cx
                .assyst()
                .http_client
                .channel(*id)
                .await
                .map_err(|e| TagParseError::TwilightHttp(Box::new(e)))?
                .model()
                .await
                .map_err(|e| TagParseError::TwilightDeserialize(Box::new(e)))?;

            Ok(Channel(channel))
        } else {
            Err(TagParseError::MismatchedCommandOptionType((
                "Channel".to_owned(),
                word.clone(),
            )))
        }
    }

    fn as_command_option(name: &str) -> CommandOption {
        ChannelBuilder::new(name, "channel argument").required(true).build()
    }
}

/// The rest of a message as an argument. This should be the last argument if used.
#[derive(Debug)]
pub struct Rest(pub String);

impl ParseArgument for Rest {
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        if let Ok(r) = ctxt.rest(label.clone()) {
            Ok(Self(r))
        } else if let Some(m) = ctxt.cx.data.message {
            if let Some(ref r) = m.referenced_message {
                Ok(Self(r.content.clone()))
            } else {
                Ok(Self(ctxt.rest(label)?))
            }
        } else {
            Ok(Self(ctxt.rest(label)?))
        }
    }

    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        // treat Rest as same as Word because there is no option type which is just one
        // whitespace-delimited word
        let word = &ctxt.option_by_name(&label.unwrap().0)?.value;

        if let CommandOptionValue::String(ref option) = word {
            Ok(Rest(option.clone()))
        } else {
            Err(TagParseError::MismatchedCommandOptionType((
                "String (Rest)".to_owned(),
                word.clone(),
            )))
        }
    }

    fn as_command_option(name: &str) -> CommandOption {
        StringBuilder::new(name, "text input").required(true).build()
    }

    fn usage(name: &str) -> String {
        format!("<...{name}>")
    }
}

/// The rest of a message as an argument, and asserts that there is no following flag argument. This
/// should be the last argument if used.
#[derive(Debug)]
pub struct RestNoFlags(pub String);

impl ParseArgument for RestNoFlags {
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        let all = ctxt.rest_all(label.clone());
        if !all.is_empty() {
            Ok(Self(all))
        } else if let Some(m) = ctxt.cx.data.message
            && let Some(ref r) = m.referenced_message
        {
            Ok(Self(r.content.clone()))
        } else {
            Err(TagParseError::ArgsExhausted(ArgsExhausted(label)))
        }
    }

    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        // treat Rest as same as Word because there is no option type which is just one
        // whitespace-delimited word
        let word = &ctxt.option_by_name(&label.unwrap().0)?.value;

        if let CommandOptionValue::String(ref option) = word {
            Ok(RestNoFlags(option.clone()))
        } else {
            Err(TagParseError::MismatchedCommandOptionType((
                "String (Rest)".to_owned(),
                word.clone(),
            )))
        }
    }

    fn as_command_option(name: &str) -> CommandOption {
        StringBuilder::new(name, "text input").required(true).build()
    }

    fn usage(name: &str) -> String {
        format!("<...{name}>")
    }
}

pub struct ImageUrl(pub String);

impl ImageUrl {
    async fn from_mention_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        let word = ctxt.next_word(label)?;

        let user_id = id_from_mention(word).ok_or(TagParseError::NoMention)?;

        if user_id == 0 {
            return Err(TagParseError::NoMention);
        }

        let user = ctxt
            .cx
            .assyst()
            .http_client
            .user(Id::new(user_id))
            .await?
            .model()
            .await?;

        Ok(Self(get_avatar_url(&user)))
    }

    async fn from_mention_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let word = &ctxt.option_by_name(&label.unwrap().0)?.value;

        if let CommandOptionValue::String(ref option) = word {
            let user_id = id_from_mention(option).ok_or(TagParseError::NoMention)?;

            if user_id == 0 {
                return Err(TagParseError::NoMention);
            }

            let user = ctxt
                .cx
                .assyst()
                .http_client
                .user(Id::new(user_id))
                .await?
                .model()
                .await?;

            Ok(Self(get_avatar_url(&user)))
        } else {
            Err(TagParseError::MismatchedCommandOptionType((
                "String (mention aregument)".to_owned(),
                word.clone(),
            )))
        }
    }

    async fn from_url_argument_raw_message(
        ctxt: &mut RawMessageParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let word = ctxt.next_word(label)?;

        if regex::URL.is_match(word) {
            Ok(Self(word.to_owned()))
        } else {
            Err(TagParseError::NoUrl)
        }
    }

    async fn from_url_argument_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let word = &ctxt.option_by_name(&label.unwrap().0)?.value;

        if let CommandOptionValue::String(ref option) = word {
            if regex::URL.is_match(option) {
                Ok(Self(option.to_owned()))
            } else {
                Err(TagParseError::NoUrl)
            }
        } else {
            Err(TagParseError::MismatchedCommandOptionType((
                "String (url argument)".to_owned(),
                word.clone(),
            )))
        }
    }

    fn attachment(attachment: Option<&Attachment>) -> Result<Self, TagParseError> {
        let attachment = attachment.ok_or(TagParseError::NoAttachment)?;
        Ok(Self(attachment.url.clone()))
    }

    async fn from_attachment_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, _: Label) -> Result<Self, TagParseError> {
        Self::attachment(ctxt.cx.data.message.as_ref().unwrap().attachments.first())
    }

    async fn from_attachment_interaction_command(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let word = &ctxt.option_by_name(&label.unwrap().0)?.value;

        if let CommandOptionValue::Attachment(ref option) = word {
            let attachment = ctxt.cx.data.interaction_attachments.get(option);
            let attachment = attachment.ok_or(TagParseError::NoAttachment)?;
            Self::attachment(Some(attachment))
        } else {
            Err(TagParseError::MismatchedCommandOptionType((
                "Attachment".to_owned(),
                word.clone(),
            )))
        }
    }

    /// This only exists for raw message
    async fn from_reply(ctxt: &mut RawMessageParseCtxt<'_>, _: Label) -> Result<Self, TagParseError> {
        let reply = ctxt
            .cx
            .data
            .message
            .as_ref()
            .unwrap()
            .referenced_message
            .as_deref()
            .ok_or(TagParseError::NoReply)?;

        if let Some(attachment) = reply.attachments.first() {
            return Ok(Self(attachment.url.clone()));
        }

        macro_rules! handle {
            ($v:expr) => {
                match $v {
                    Ok(v) => return Ok(v),
                    Err(err) if err.get_severity() == ErrorSeverity::High => return Err(err),
                    _ => {},
                }
            };
        }

        handle!(Self::sticker(reply.sticker_items.first()));
        handle!(Self::embed(reply.embeds.first()));
        handle!(Self::emoji(&mut ctxt.cx, &reply.content).await);

        Err(TagParseError::NoReply)
    }

    fn embed(embed: Option<&Embed>) -> Result<Self, TagParseError> {
        let embed = embed.ok_or(TagParseError::NoEmbed)?;

        if let Some(url) = &embed.url
            && url.starts_with("https://tenor.com/view/")
        {
            return Ok(Self(url.clone()));
        }

        if let Some(image) = &embed.image {
            return Ok(Self(image.url.clone()));
        }

        if let Some(thumbnail) = &embed.thumbnail {
            return Ok(Self(thumbnail.url.clone()));
        }

        if let Some(video) = &embed.video
            && let Some(url) = &video.url
        {
            Ok(Self(url.clone()))
        } else {
            Err(TagParseError::NoEmbed)
        }
    }

    async fn emoji(ctxt: &mut CommandCtxt<'_>, word: &str) -> Result<Self, TagParseError> {
        #[derive(Deserialize)]
        struct TwemojiVendorImage {
            pub twitter: String,
        }

        #[derive(Deserialize)]
        struct TwemojiLookup {
            pub vendor_images: TwemojiVendorImage,
        }

        if let Some(e) = emoji::lookup_by_glyph::lookup(word) {
            let codepoint = e.codepoint.to_lowercase().replace(' ', "-").replace("-fe0f", "");

            let emoji_url = format!("https://bignutty.gitlab.io/emojipedia-data/data/{codepoint}.json");
            let dl = ctxt
                .assyst()
                .reqwest_client
                .get(emoji_url)
                .send()
                .await?
                .json::<TwemojiLookup>()
                .await?;

            Ok(Self(dl.vendor_images.twitter))
        } else {
            Err(TagParseError::NoEmoji)
        }
    }

    async fn from_emoji_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        let word = ctxt.next_word(label)?;
        Self::emoji(&mut ctxt.cx, word).await
    }

    async fn from_emoji_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let word = &ctxt.option_by_name(&label.unwrap().0)?.value;

        if let CommandOptionValue::String(ref option) = word {
            if regex::URL.is_match(option) {
                Ok(Self::emoji(&mut ctxt.cx, option).await?)
            } else {
                Err(TagParseError::NoUrl)
            }
        } else {
            Err(TagParseError::MismatchedCommandOptionType((
                "String (emoji argument)".to_owned(),
                word.clone(),
            )))
        }
    }

    fn sticker(sticker: Option<&MessageSticker>) -> Result<Self, TagParseError> {
        let sticker = sticker.ok_or(TagParseError::NoSticker)?;
        match sticker.format_type {
            StickerFormatType::Png => Ok(Self(format!("https://cdn.discordapp.com/stickers/{}.png", sticker.id))),
            _ => Err(TagParseError::UnsupportedSticker(sticker.format_type)),
        }
    }

    /// This only exists for raw message
    async fn from_sticker(ctxt: &mut RawMessageParseCtxt<'_>, _: Label) -> Result<Self, TagParseError> {
        Self::sticker(ctxt.cx.data.message.as_ref().unwrap().sticker_items.first())
    }

    // Defined separately without a CommandCtxt because it is also used elsewhere where we don't
    // have one (and this also doesn't need it)
    pub async fn from_channel_history(
        assyst: &Assyst,
        channel_id: Id<ChannelMarker>,
    ) -> Result<ImageUrl, TagParseError> {
        let messages = match assyst.http_client.channel_messages(channel_id).await {
            Ok(m) => m.models().await?,
            Err(_) => return Err(TagParseError::FailedToGetMessageHistory),
        };

        macro_rules! handle {
            ($v:expr) => {
                // Ignore any error, even high severity ones, since not doing that would mean
                // we bail when we see a "random" malformed message in a channel
                if let Ok(v) = $v {
                    return Ok(v);
                }
            };
        }

        for message in messages {
            handle!(Self::embed(message.embeds.first()));
            handle!(Self::sticker(message.sticker_items.first()));
            handle!(Self::sticker(message.sticker_items.first()));
            handle!(Self::attachment(message.attachments.first()));
        }

        Err(TagParseError::NoImageInHistory)
    }
}

impl Display for ImageUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ParseArgument for ImageUrl {
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        async fn combined_parsers(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<ImageUrl, TagParseError> {
            macro_rules! handle {
                ($v:expr) => {
                    match $v {
                        Ok(r) => return Ok(r),
                        Err(err) if let TagParseError::TwilightHttp(_) = err => {},
                        Err(err) if err.get_severity() == ErrorSeverity::High => return Err(err),
                        _ => {},
                    }
                };
            }

            handle!(commit_if_ok!(ctxt, ImageUrl::from_mention_raw_message, label));
            handle!(commit_if_ok!(ctxt, ImageUrl::from_url_argument_raw_message, label));
            handle!(commit_if_ok!(ctxt, ImageUrl::from_attachment_raw_message, label));
            handle!(commit_if_ok!(ctxt, ImageUrl::from_reply, label));
            handle!(commit_if_ok!(ctxt, ImageUrl::from_emoji_raw_message, label));
            handle!(commit_if_ok!(ctxt, ImageUrl::from_sticker, label));
            if !ctxt.cx.data.command_from_install_context {
                handle!(ImageUrl::from_channel_history(ctxt.cx.assyst(), ctxt.cx.data.channel_id).await)
            } else {
                return Err(TagParseError::MessageHistoryUnavailableInContext);
            };
            Err(TagParseError::NoImageFound)
        }

        let ImageUrl(mut url) = combined_parsers(ctxt, label).await?;

        // tenor urls only typically return a png, so this code visits the url
        // and extracts the appropriate GIF url from the page.
        if url.starts_with("https://tenor.com/view") {
            let page = ctxt.cx.assyst().reqwest_client.get(&url).send().await?.text().await?;

            let gif_url = regex::TENOR_GIF.find(&page).ok_or(TagParseError::MediaDownloadFail)?;
            url = gif_url.as_str().to_owned();
        }

        Ok(Self(url))
    }

    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        async fn combined_parsers(
            ctxt: &mut InteractionCommandParseCtxt<'_>,
            label: Label,
        ) -> Result<ImageUrl, TagParseError> {
            macro_rules! handle {
                ($v:expr) => {
                    match $v {
                        Ok(r) => return Ok(r),
                        Err(err) if err.get_severity() == ErrorSeverity::High => return Err(err),
                        _ => {},
                    }
                };
            }

            handle!(commit_if_ok!(
                ctxt,
                ImageUrl::from_attachment_interaction_command,
                label
            ));
            handle!(commit_if_ok!(ctxt, ImageUrl::from_mention_command_option, label));
            handle!(commit_if_ok!(ctxt, ImageUrl::from_url_argument_command_option, label));
            handle!(commit_if_ok!(ctxt, ImageUrl::from_emoji_command_option, label));
            handle!(ImageUrl::from_channel_history(ctxt.cx.assyst(), ctxt.cx.data.channel_id).await);
            Err(TagParseError::NoImageFound)
        }

        let ImageUrl(mut url) = combined_parsers(ctxt, label).await?;

        // tenor urls only typically return a png, so this code visits the url
        // and extracts the appropriate GIF url from the page.
        if url.starts_with("https://tenor.com/view") {
            let page = ctxt.cx.assyst().reqwest_client.get(&url).send().await?.text().await?;

            let gif_url = regex::TENOR_GIF.find(&page).ok_or(TagParseError::MediaDownloadFail)?;
            url = gif_url.as_str().to_owned();
        }

        Ok(Self(url))
    }

    fn as_command_option(name: &str) -> CommandOption {
        AttachmentBuilder::new(name, "attachment input").required(false).build()
    }
}

pub struct Image(pub Vec<u8>);

impl ParseArgument for Image {
    async fn parse_raw_message(ctxt: &mut RawMessageParseCtxt<'_>, label: Label) -> Result<Self, TagParseError> {
        let ImageUrl(url) = ImageUrl::parse_raw_message(ctxt, label).await?;

        let data = downloader::download_content(
            &ctxt.cx.assyst().reqwest_client,
            &url,
            ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES,
            true,
        )
        .await?;
        Ok(Image(data))
    }

    async fn parse_command_option(
        ctxt: &mut InteractionCommandParseCtxt<'_>,
        label: Label,
    ) -> Result<Self, TagParseError> {
        let ImageUrl(url) = ImageUrl::parse_command_option(ctxt, label).await?;

        let data = downloader::download_content(
            &ctxt.cx.assyst().reqwest_client,
            &url,
            ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES,
            true,
        )
        .await?;
        Ok(Image(data))
    }

    fn as_command_option(name: &str) -> CommandOption {
        AttachmentBuilder::new(name, "attachment input").required(false).build()
    }
}
