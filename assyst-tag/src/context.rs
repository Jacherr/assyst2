use anyhow::anyhow;
use assyst_common::eval::FakeEvalImageResponse;

/// A "no-op" context, which returns an error for any of the methods
///
/// This is useful for testing the parser, when you need to provide a Context but
/// don't really need its functionality.
pub struct NopContext;

fn not_implemented<T>() -> anyhow::Result<T> {
    Err(anyhow!("Not implemented"))
}

/// External context for the parser
///
/// It contains methods that can be provided by the caller (normally the bot crate).
pub trait Context {
    /// Executes provided JavaScript code and returns the result (string or image)
    fn execute_javascript(&self, code: &str, args: Vec<String>) -> anyhow::Result<FakeEvalImageResponse>;
    /// Returns the URL of the last attachment
    fn get_last_attachment(&self) -> anyhow::Result<String>;
    /// Returns the avatar URL of the provided user, or the message author
    fn get_avatar(&self, user_id: Option<u64>) -> anyhow::Result<String>;
    /// Downloads the URL and returns the contents as a string
    fn download(&self, url: &str) -> anyhow::Result<String>;
    /// Returns the channel ID of where this message was sent
    fn channel_id(&self) -> anyhow::Result<u64>;
    /// Returns the guild ID of where this message was sent
    fn guild_id(&self) -> anyhow::Result<u64>;
    /// Returns the user ID of the message author
    fn user_id(&self) -> anyhow::Result<u64>;
    /// Returns the tag of the provided ID
    fn user_tag(&self, id: Option<u64>) -> anyhow::Result<String>;
    /// Loads the contents of a tag
    fn get_tag_contents(&self, tag: &str) -> anyhow::Result<String>;
}

impl Context for NopContext {
    fn execute_javascript(&self, _code: &str, _args: Vec<String>) -> anyhow::Result<FakeEvalImageResponse> {
        not_implemented()
    }

    fn get_last_attachment(&self) -> anyhow::Result<String> {
        not_implemented()
    }

    fn get_avatar(&self, _user_id: Option<u64>) -> anyhow::Result<String> {
        not_implemented()
    }

    fn download(&self, _url: &str) -> anyhow::Result<String> {
        not_implemented()
    }

    fn channel_id(&self) -> anyhow::Result<u64> {
        not_implemented()
    }

    fn guild_id(&self) -> anyhow::Result<u64> {
        not_implemented()
    }

    fn user_id(&self) -> anyhow::Result<u64> {
        not_implemented()
    }

    fn user_tag(&self, _id: Option<u64>) -> anyhow::Result<String> {
        not_implemented()
    }

    fn get_tag_contents(&self, _: &str) -> anyhow::Result<String> {
        not_implemented()
    }
}

impl Context for &dyn Context {
    fn execute_javascript(&self, code: &str, args: Vec<String>) -> anyhow::Result<FakeEvalImageResponse> {
        (**self).execute_javascript(code, args)
    }

    fn get_last_attachment(&self) -> anyhow::Result<String> {
        (**self).get_last_attachment()
    }

    fn get_avatar(&self, user_id: Option<u64>) -> anyhow::Result<String> {
        (**self).get_avatar(user_id)
    }

    fn download(&self, url: &str) -> anyhow::Result<String> {
        (**self).download(url)
    }

    fn channel_id(&self) -> anyhow::Result<u64> {
        (**self).channel_id()
    }

    fn guild_id(&self) -> anyhow::Result<u64> {
        (**self).guild_id()
    }

    fn user_id(&self) -> anyhow::Result<u64> {
        (**self).user_id()
    }

    fn user_tag(&self, user_id: Option<u64>) -> anyhow::Result<String> {
        (**self).user_tag(user_id)
    }

    fn get_tag_contents(&self, tag: &str) -> anyhow::Result<String> {
        (**self).get_tag_contents(tag)
    }
}
