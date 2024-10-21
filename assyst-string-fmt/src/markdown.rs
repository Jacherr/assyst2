use std::fmt::Display;

use regex::{Captures, Regex};

pub trait Markdown {
    fn escape_italics(&self) -> String;
    fn escape_bold(&self) -> String;
    fn escape_codestring(&self) -> String;
    fn escape_codeblock(&self, language: impl Display) -> String;
    fn escape_spoiler(&self) -> String;
    fn escape_strikethrough(&self) -> String;
    fn escape_underline(&self) -> String;

    fn italics(&self) -> String;
    fn bold(&self) -> String;
    fn codestring(&self) -> String;
    fn codeblock(&self, language: impl Display) -> String;
    fn spoiler(&self) -> String;
    fn strikethrough(&self) -> String;
    fn underline(&self) -> String;
    fn url(&self, url: impl Display, comment: Option<impl Display>) -> String;
    fn timestamp(seconds: usize, style: TimestampStyle) -> String {
        format!("<t:{seconds}:{style}>")
    }
    fn subtext(&self) -> String;
}

pub enum TimestampStyle {
    FullLong,
    FullShort,
    DateLong,
    DateShort,
    TimeLong,
    TimeShort,
    Relative,
}

impl Display for TimestampStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::FullLong => "F",
                Self::FullShort => "f",
                Self::DateLong => "D",
                Self::DateShort => "d",
                Self::TimeLong => "T",
                Self::TimeShort => "t",
                Self::Relative => "R",
            }
        )
    }
}

fn cut(t: impl Display, to: usize) -> String {
    t.to_string().chars().take(to).collect::<String>()
}

impl<T> Markdown for T
where
    T: Display,
{
    fn escape_italics(&self) -> String {
        Regex::new(r"_|\*")
            .unwrap()
            .replace_all(&cut(self, 1998), |x: &Captures| {
                format!("\\{}", x.get(0).unwrap().as_str())
            })
            .into_owned()
    }
    fn escape_bold(&self) -> String {
        Regex::new(r"\*\*")
            .unwrap()
            .replace_all(&cut(self, 1998), r"\*\*")
            .into_owned()
    }
    fn escape_codestring(&self) -> String {
        Regex::new(r"`")
            .unwrap()
            .replace_all(&cut(self, 1998), r"'")
            .into_owned()
    }
    fn escape_codeblock(&self, language: impl Display) -> String {
        Regex::new(r"```")
            .unwrap()
            .replace_all(&cut(self, 1988 - language.to_string().len()), "`\u{200b}`\u{200b}`")
            .into_owned()
    }

    fn escape_spoiler(&self) -> String {
        Regex::new(r"__")
            .unwrap()
            .replace_all(&cut(self, 1996), r"\|\|")
            .into_owned()
    }
    fn escape_strikethrough(&self) -> String {
        Regex::new(r"~~")
            .unwrap()
            .replace_all(&cut(self, 1996), r"\~\~")
            .into_owned()
    }
    fn escape_underline(&self) -> String {
        Regex::new(r"\|\|")
            .unwrap()
            .replace_all(&cut(self, 1996), r"\_\_")
            .into_owned()
    }

    fn italics(&self) -> String {
        format!("_{}_", self.escape_italics())
    }

    fn bold(&self) -> String {
        format!("**{}**", self.escape_bold())
    }

    fn codestring(&self) -> String {
        format!("`{}`", self.escape_codestring())
    }

    fn codeblock(&self, language: impl Display) -> String {
        let t = self.escape_codeblock(&language);
        format!("```{language}\n{t}\n```")
    }

    fn spoiler(&self) -> String {
        format!("||{}||", self.escape_italics())
    }

    fn strikethrough(&self) -> String {
        format!("~~{}~~", self.escape_italics())
    }

    fn underline(&self) -> String {
        format!("__{}__", self.escape_italics())
    }

    fn url(&self, url: impl Display, comment: Option<impl Display>) -> String {
        format!(
            "[{self}]({url}{})",
            match comment {
                Some(c) => format!(" '{c}'"),
                None => "".to_string(),
            }
        )
    }

    fn subtext(&self) -> String {
        format!("-# {self}")
    }
}

pub fn parse_codeblock(input: String) -> String {
    if input.trim().starts_with("```") && input.trim().ends_with("```") {
        let r = input.trim().replace("\n", "\n ");
        let new = r.split(" ").skip(1).collect::<Vec<_>>();
        let joined = new.join(" ");
        joined[..joined.len() - 3].to_owned()
    } else if input.trim().starts_with("`") && input.trim().ends_with("`") {
        input[1..input.len() - 1].to_owned()
    } else {
        input
    }
}
