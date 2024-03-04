use std::fmt::Display;
use std::ops::Deref;

use assyst_common::util::discord::get_avatar_url;
use assyst_common::util::{parse_to_millis, regex};
use serde::Deserialize;
use twilight_model::channel::message::sticker::{MessageSticker, StickerFormatType};
use twilight_model::channel::message::Embed;
use twilight_model::channel::Attachment;
use twilight_model::id::Id;

use crate::downloader::{self, ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES};
use crate::gateway_handler::message_parser::error::{ErrorSeverity, GetErrorSeverity};

use super::errors::TagParseError;
use super::CommandCtxt;

pub trait ParseArgument: Sized {
    /// Parses `Self`, given a command.
    async fn parse(ctxt: &mut CommandCtxt<'_>) -> Result<Self, TagParseError>;
}

// impl for number types
macro_rules! ii_this {
    ($($t:path)*) => {
        $(
            impl ParseArgument for $t {
                async fn parse(ctxt: &mut CommandCtxt<'_>) -> Result<Self, TagParseError> {
                    let word = ctxt.next_word()?;
                    Ok(word.parse()?)
                }
            }
        )*
    };
}

ii_this!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize f64 f32);

impl<T: ParseArgument> ParseArgument for Option<T> {
    async fn parse(ctxt: &mut CommandCtxt<'_>) -> Result<Self, TagParseError> {
        // TODO: should we be using commit_if_ok to undo failed parsers?
        match T::parse(ctxt).await {
            Ok(v) => Ok(Some(v)),
            Err(err) if err.get_severity() == ErrorSeverity::High => Err(err),
            _ => Ok(None),
        }
    }
}

impl<T: ParseArgument> ParseArgument for Vec<T> {
    async fn parse(ctxt: &mut CommandCtxt<'_>) -> Result<Self, TagParseError> {
        let mut items = Vec::new();

        // `Option<T>`'s parser takes care of recovering from low severity errors
        // and any `Err`s returned are fatal, so we can just use `?`
        while let Some(value) = <Option<T>>::parse(ctxt).await? {
            items.push(value)
        }

        Ok(items)
    }
}

/// A time argument such as `1h20m30s`.
#[derive(Debug)]
pub struct Time {
    pub millis: u64,
}

impl ParseArgument for Time {
    async fn parse(ctxt: &mut CommandCtxt<'_>) -> Result<Self, TagParseError> {
        let word = ctxt.next_word()?;
        let millis = parse_to_millis(word)?;

        Ok(Time { millis })
    }
}

/// A single word argument.
#[derive(Debug)]
pub struct Word(pub String);

impl ParseArgument for Word {
    async fn parse(ctxt: &mut CommandCtxt<'_>) -> Result<Self, TagParseError> {
        Ok(Self(ctxt.next_word()?.to_owned()))
    }
}

/// The rest of a message as an argument. This should be the last argument if used.
#[derive(Debug)]
pub struct Rest(pub String);

impl ParseArgument for Rest {
    async fn parse(ctxt: &mut CommandCtxt<'_>) -> Result<Self, TagParseError> {
        Ok(Self(ctxt.rest()?.to_owned()))
    }
}

#[derive(Debug)]
pub struct Removable<T>(pub Option<T>);

impl<T> Deref for Removable<T> {
    type Target = Option<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
pub const NONE_STR: &str = "_"; // change if `_` is undesirable.
impl<T> ParseArgument for Removable<T>
where
    T: ParseArgument,
{
    async fn parse(ctxt: &mut CommandCtxt<'_>) -> Result<Self, TagParseError> {
        let word = ctxt.next_word()?;

        if word == NONE_STR {
            Ok(Self(None))
        } else {
            Ok(Self(Some(T::parse(ctxt).await?)))
        }
    }
}

pub type DesiredCmpTy = isize;

macro_rules! cmp_arg {
    ($($label:ident $l:literal => $op:tt,)*) => {
        $(
            pub struct $label<const N: DesiredCmpTy>(DesiredCmpTy);

            impl<const N: DesiredCmpTy> ParseArgument for $label<N> {
                async fn parse(ctxt: &mut CommandCtxt<'_>) -> Result<Self, TagParseError> {
                    let value = DesiredCmpTy::parse(ctxt).await?;

                    if value $op N { Ok(Self(value)) } else { Err(TagParseError::ComparisonError(value, N, $l.to_string())) }
                }
            }
        )*
    };
}

cmp_arg!(
    Gt "greater than" => >,
    Ge "greater than or equal to" => >=,
    Lt "less than" => <,
    Le "less than or equal to" => <=,
    Eq "equal to" => ==, // redundant, probably
    Ne "not equal to" => !=,
);

pub struct Ranged<const A: DesiredCmpTy, const B: DesiredCmpTy>(DesiredCmpTy);

impl<const A: DesiredCmpTy, const B: DesiredCmpTy> ParseArgument for Ranged<A, B> {
    async fn parse(ctxt: &mut CommandCtxt<'_>) -> Result<Self, TagParseError> {
        let value = DesiredCmpTy::parse(ctxt).await?;

        if A <= value && value <= B {
            Ok(Self(value))
        } else { 
            Err(TagParseError::RangeError(value, A, B))
        }
    }
}
        
// TODO: @trueharuu
// pub struct Flag<const LABEL: &'static str, T>(Option<T>);
// pub const FLAG_PREFIX: &str = "-";

// impl<const LABEL: &'static str, T> ParseArgument for Flag<LABEL, T>
// where
//     T: ParseArgument,
// {
//     async fn parse(ctxt: &mut CommandCtxt<'_>) -> Result<Self, TagParseError> {
//         let mut f = ctxt.fork();
//         let label = f.next_word()?;
//         if label.strip_prefix(FLAG_PREFIX) == Some(LABEL) {
//             Ok(Self(Some(T::parse(&mut f).await?)))
//         } else {
//             // there was no flag present.
//             Ok(Self(None))
//         }
//     }
// }

pub struct ImageUrl(String);
impl ImageUrl {
    async fn from_mention(mut ctxt: CommandCtxt<'_>) -> Result<(Self, CommandCtxt<'_>), TagParseError> {
        let word = ctxt.next_word()?;

        let user_id = regex::USER_MENTION
            .captures(word)
            .and_then(|user_id_capture| user_id_capture.get(1))
            .map(|id| id.as_str())
            .and_then(|id| id.parse::<u64>().ok())
            .ok_or(TagParseError::NoMention)?;

        if user_id == 0 {
            return Err(TagParseError::NoMention);
        }

        let user = ctxt.assyst().http_client.user(Id::new(user_id)).await?.model().await?;

        Ok((Self(get_avatar_url(&user)), ctxt))
    }

    async fn from_url_argument(mut ctxt: CommandCtxt<'_>) -> Result<(Self, CommandCtxt<'_>), TagParseError> {
        let word = ctxt.next_word()?;

        if regex::URL.is_match(word) {
            Ok((Self(word.to_owned()), ctxt))
        } else {
            Err(TagParseError::NoUrl)
        }
    }

    fn attachment(attachment: Option<&Attachment>) -> Result<Self, TagParseError> {
        let attachment = attachment.ok_or(TagParseError::NoAttachment)?;
        Ok(Self(attachment.url.clone()))
    }

    async fn from_attachment(ctxt: CommandCtxt<'_>) -> Result<(Self, CommandCtxt<'_>), TagParseError> {
        Self::attachment(ctxt.data.attachment).map(|a| (a, ctxt))
    }

    async fn from_reply(mut ctxt: CommandCtxt<'_>) -> Result<(Self, CommandCtxt<'_>), TagParseError> {
        let reply = ctxt.data.referenced_message.ok_or(TagParseError::NoReply)?; // TODO: not args exhausted

        if let Some(attachment) = reply.attachments.first() {
            return Ok((Self(attachment.url.clone()), ctxt));
        }

        macro_rules! handle {
            ($v:expr) => {
                match $v {
                    Ok(v) => return Ok((v, ctxt)),
                    Err(err) if err.get_severity() == ErrorSeverity::High => return Err(err),
                    _ => {},
                }
            };
        }

        handle!(Self::sticker(reply.sticker_items.first()));
        handle!(Self::embed(reply.embeds.first()));
        handle!(Self::emoji(&mut ctxt, &reply.content).await);

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

            let emoji_url = format!("https://bignutty.gitlab.io/emojipedia-data/data/{}.json", codepoint);
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

    async fn from_emoji(mut ctxt: CommandCtxt<'_>) -> Result<(Self, CommandCtxt<'_>), TagParseError> {
        let word = ctxt.next_word()?;
        Self::emoji(&mut ctxt, word).await.map(|e| (e, ctxt))
    }

    fn sticker(sticker: Option<&MessageSticker>) -> Result<Self, TagParseError> {
        let sticker = sticker.ok_or(TagParseError::NoSticker)?;
        match sticker.format_type {
            StickerFormatType::Png => Ok(Self(format!("https://cdn.discordapp.com/stickers/{}.png", sticker.id))),
            _ => Err(TagParseError::UnsupportedSticker(sticker.format_type)),
        }
    }

    async fn from_sticker(ctxt: CommandCtxt<'_>) -> Result<(Self, CommandCtxt<'_>), TagParseError> {
        Self::sticker(ctxt.data.sticker).map(|v| (v, ctxt))
    }

    async fn from_channel_history(ctxt: CommandCtxt<'_>) -> Result<(Self, CommandCtxt<'_>), TagParseError> {
        let messages = ctxt
            .assyst()
            .http_client
            .channel_messages(Id::new(ctxt.data.channel_id))
            .await?
            .models()
            .await?;

        macro_rules! handle {
            ($v:expr) => {
                // Ignore any error, even high severity ones, since not doing that would mean
                // we bail when we see a "random" malformed message in a channel
                if let Ok(v) = $v {
                    return Ok((v, ctxt));
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
    async fn parse(ctxt: &mut CommandCtxt<'_>) -> Result<Self, TagParseError> {
        async fn combined_parsers(ctxt: &mut CommandCtxt<'_>) -> Result<ImageUrl, TagParseError> {
            macro_rules! handle {
                ($v:expr) => {
                    match $v {
                        Ok(r) => return Ok(r),
                        Err(err) if err.get_severity() == ErrorSeverity::High => return Err(err),
                        _ => {},
                    }
                };
            }

            handle!(ctxt.commit_if_ok(ImageUrl::from_mention).await);
            handle!(ctxt.commit_if_ok(ImageUrl::from_url_argument).await);
            handle!(ctxt.commit_if_ok(ImageUrl::from_attachment).await);
            handle!(ctxt.commit_if_ok(ImageUrl::from_reply).await);
            handle!(ctxt.commit_if_ok(ImageUrl::from_emoji).await);
            handle!(ctxt.commit_if_ok(ImageUrl::from_sticker).await);
            handle!(ctxt.commit_if_ok(ImageUrl::from_channel_history).await);
            Err(TagParseError::NoImageFound)
        }

        let ImageUrl(mut url) = combined_parsers(ctxt).await?;

        // tenor urls only typically return a png, so this code visits the url
        // and extracts the appropriate GIF url from the page.
        if url.starts_with("https://tenor.com/view") {
            let page = ctxt.assyst().reqwest_client.get(&url).send().await?.text().await?;

            let gif_url = regex::TENOR_GIF.find(&page).ok_or(TagParseError::MediaDownloadFail)?;
            url = gif_url.as_str().to_owned();
        }

        Ok(Self(url))
    }
}

pub struct Image(pub Vec<u8>);

impl ParseArgument for Image {
    async fn parse(ctxt: &mut CommandCtxt<'_>) -> Result<Self, TagParseError> {
        let ImageUrl(url) = ImageUrl::parse(ctxt).await?;

        let data = downloader::download_content(ctxt.assyst(), &url, ABSOLUTE_INPUT_FILE_SIZE_LIMIT_BYTES).await?;
        Ok(Image(data))
    }
}
