use assyst_common::util::filetype::{get_sig, Type};

use super::arguments::Image;

#[derive(Debug)]
pub struct Attachment {
    pub name: Box<str>,
    pub data: Vec<u8>,
}

impl From<Image> for Attachment {
    fn from(value: Image) -> Self {
        let ext = get_sig(&value.0).unwrap_or_else(|| Type::PNG).as_str();
        Attachment {
            name: format!("attachment.{ext}").into(),
            data: value.0,
        }
    }
}

#[derive(Debug)]
pub struct MessageBuilder {
    pub content: Option<String>,
    pub attachment: Option<Attachment>,
}

impl From<&str> for MessageBuilder {
    fn from(value: &str) -> Self {
        Self {
            content: Some(value.into()),
            attachment: None,
        }
    }
}
impl From<String> for MessageBuilder {
    fn from(value: String) -> Self {
        Self {
            content: Some(value),
            attachment: None,
        }
    }
}

impl From<Attachment> for MessageBuilder {
    fn from(value: Attachment) -> Self {
        Self {
            content: None,
            attachment: Some(value),
        }
    }
}

impl From<Image> for MessageBuilder {
    fn from(value: Image) -> Self {
        Self {
            content: None,
            attachment: Some(value.into()),
        }
    }
}
impl From<(Image, &str)> for MessageBuilder {
    fn from((image, text): (Image, &str)) -> Self {
        Self {
            attachment: Some(image.into()),
            content: Some(text.into()),
        }
    }
}
