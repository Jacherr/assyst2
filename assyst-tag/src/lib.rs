#![warn(rust_2018_idioms)]
#![feature(round_char_boundary, if_let_guard)]

use std::cell::RefCell;
use std::collections::HashMap;

use assyst_common::util::filetype::Type;
pub use context::{Context, NopContext};
use errors::TResult;
use parser::{Counter, ParseMode, Parser, SharedState};

mod context;
pub mod errors;
pub mod parser;
mod subtags;

#[derive(Debug)]
pub struct ParseResult {
    pub output: String,
    pub attachment: Option<(Vec<u8>, Type)>,
}

pub fn parse<C: Context>(input: &str, args: &[&str], mode: ParseMode, cx: C) -> TResult<ParseResult> {
    let variables = RefCell::new(HashMap::new());
    let counter = Counter::default();
    let attachment = RefCell::new(None);
    let state = SharedState::new(&variables, &counter, &attachment);

    let output = Parser::new(input.as_bytes(), args, state, mode, &cx).parse_segment(true)?;

    Ok(ParseResult {
        output,
        attachment: attachment.into_inner(),
    })
}

/// NOTE: be careful when bubbling up potential errors -- you most likely want to wrap them in
/// `ErrorKind::Nested`
pub fn parse_with_parent(input: &str, parent: &Parser<'_>, side_effects: bool) -> TResult<String> {
    Parser::from_parent(input.as_bytes(), parent).parse_segment(side_effects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorKind;

    macro_rules! test {
        ($mode:expr; $( $name:ident: $input:expr => $result:pat ),+ $(,)?) => {
            $(
            #[test]
                fn $name() {
                    let input = $input;
                    let res = parse(input, &[], $mode, NopContext);
                    assert!(matches!(res.as_ref().map_err(|err| &*err.kind).map(|ok| &*ok.output), $result));
                    if let Err(err) = res {

                        // try formatting it to find any potential panic bugs
                        errors::format_error(input, err);
                    }
                }
            )*
        };
    }

    test!(ParseMode::StopOnError;
        crash1: "zzzz@z{z" => Err(ErrorKind::MissingClosingBrace { .. }),
        crash2: "{if|z|~|\u{7}|gs|I---s{args|" => Err(ErrorKind::MissingClosingBrace{..}),
        crash3: "{a:a" => Err(ErrorKind::MissingClosingBrace{..}),
        crash4: "{note:::::JJJJ:::::::::::)[x{||z\0\0{z|||{i:||||{\u{7}Ƅ" => Err(ErrorKind::MissingClosingBrace { .. }),
        crash5: "{max}Ӱ______________________________" => Err(ErrorKind::ArgParseError { .. }),
        crash6: "{args}ӌs" => Ok("ӌs"),
        iter_limit: &"{max:0}".repeat(501) => Err(ErrorKind::IterLimit{..}),
        if_then_works: "{if:{argslen}|=|0|ok|wrong}" => Ok("ok"),
        if_else_works: "{if:{argslen}|=|1|wrong|ok}" => Ok("ok"),
        separator_outside_tag: "a|" => Ok("a|"),
        separator_outside_tag2: "a|}" => Ok("a|}"),
        separator_outside_tag3: "a|b" => Ok("a|b"),
        spoiler: "a||b||c" => Ok("a||b||c"),
        spoiler2: "a||b||" => Ok("a||b||"),
        spoiler_in_subparser: "{eval:a||b||c}" => Ok("a"),
        spoiler_in_subtag: "{note:a||b||c}" => Err(ErrorKind::MissingClosingBrace { .. }),
    );

    test!(ParseMode::IgnoreOnError;
        recover1: "{foo!:42" => Ok("{foo!:42"),
        recover2: "{foo!:42}" => Ok("{foo!:42}"),
        recover3: "{foo:42}" => Ok("{foo:42}"),
        asynk: "(async () => {...})" => Ok("(async () => {...})"),
        asynk2: "(async () => { return 42 })" => Ok("(async () => { return 42 })")
    );
}
