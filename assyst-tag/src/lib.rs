#![warn(rust_2018_idioms)]

use assyst_common::util::filetype::Type;
pub use context::{Context, NopContext};
use errors::TResult;
use parser::{Counter, Parser, SharedState};
use std::cell::RefCell;
use std::collections::HashMap;

mod context;
pub mod errors;
pub mod parser;
mod subtags;

#[derive(Debug)]
pub struct ParseResult {
    pub output: String,
    pub attachment: Option<(Vec<u8>, Type)>,
}

pub fn parse<C: Context>(input: &str, args: &[&str], cx: C) -> TResult<ParseResult> {
    let variables = RefCell::new(HashMap::new());
    let counter = Counter::default();
    let attachment = RefCell::new(None);
    let state = SharedState::new(&variables, &counter, &attachment);

    let output = Parser::new(input.as_bytes(), args, state, &cx).parse_segment(true)?;

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

    #[test]
    fn parse_test() {
        // let input = "{if: {argslen} |>=| 1| {avatar:{idof:{args}}} | {avatar}}?size=4096";
        // {foo
        // {foo bar
        // {}
        // a {} b
        // a { b
        // a {
        // { testing stuff
        // {
        // { hello sdfsdfsdfsd fsdf sdh fsdh sdfj f}
        let input = "{set:a|{range:1|3}}
        {if:{get:a}|=|1|one|2|two|3|three}";
        dbg!(input.len());
        let segment = parse(input, &["a"], NopContext);
        match segment {
            Ok(r) => println!("{r:?}"),
            Err(e) => println!("{}\n\n", errors::format_error(input, dbg!(e))),
        }
        // match segment {
        //     Ok(r) => println!("{r:?}"),
        //     Err(e) => println!("Error: {:?}", e),
        // }
    }

    #[test]
    fn tags_invoke_each_other() {
        let input = "tag content: {tag:wtf|a|b}!";
        let segment = parse(input, &["h", "o"], NopContext);
        // match segment {
        //     Ok(r) => println!("{r:?}"),
        //     Err(e) => println!("Error: {:?}", e),
        // }
    }
}
