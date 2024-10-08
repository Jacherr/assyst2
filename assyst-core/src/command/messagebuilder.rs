use assyst_common::util::filetype::{get_sig, Type};
use twilight_model::channel::message::Component;

use super::arguments::Image;
use super::componentctxt::ComponentCtxtRegister;

#[derive(Debug)]
pub struct Attachment {
    pub name: Box<str>,
    pub data: Vec<u8>,
}

impl From<Image> for Attachment {
    fn from(value: Image) -> Self {
        let ext = get_sig(&value.0).unwrap_or(Type::PNG).as_str();
        Attachment {
            name: format!("attachment.{ext}").into(),
            data: value.0,
        }
    }
}

pub struct MessageBuilder {
    pub content: Option<String>,
    pub attachment: Option<Attachment>,
    pub components: Option<Vec<Component>>,
    pub component_ctxt: Option<ComponentCtxtRegister>,
}

impl From<&str> for MessageBuilder {
    fn from(value: &str) -> Self {
        Self {
            content: Some(value.into()),
            attachment: None,
            components: None,
            component_ctxt: None,
        }
    }
}
impl From<String> for MessageBuilder {
    fn from(value: String) -> Self {
        Self {
            content: Some(value),
            attachment: None,
            components: None,
            component_ctxt: None,
        }
    }
}

impl From<Attachment> for MessageBuilder {
    fn from(value: Attachment) -> Self {
        Self {
            content: None,
            attachment: Some(value),
            components: None,
            component_ctxt: None,
        }
    }
}

impl From<(Attachment, String)> for MessageBuilder {
    fn from(value: (Attachment, String)) -> Self {
        Self {
            content: Some(value.1),
            attachment: Some(value.0),
            components: None,
            component_ctxt: None,
        }
    }
}

impl From<Image> for MessageBuilder {
    fn from(value: Image) -> Self {
        Self {
            content: None,
            attachment: Some(value.into()),
            components: None,
            component_ctxt: None,
        }
    }
}
impl From<(Image, &str)> for MessageBuilder {
    fn from((image, text): (Image, &str)) -> Self {
        Self {
            attachment: Some(image.into()),
            content: Some(text.into()),
            components: None,
            component_ctxt: None,
        }
    }
}
impl From<Vec<u8>> for MessageBuilder {
    fn from(value: Vec<u8>) -> Self {
        Self {
            attachment: Some(Image(value).into()),
            content: None,
            components: None,
            component_ctxt: None,
        }
    }
}
impl From<(Vec<u8>, &str)> for MessageBuilder {
    fn from((value, text): (Vec<u8>, &str)) -> Self {
        Self {
            attachment: Some(Image(value).into()),
            content: Some(text.into()),
            components: None,
            component_ctxt: None,
        }
    }
}
