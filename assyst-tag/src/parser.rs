use crate::context::Context;
use crate::errors::{err_res, BytePos, ErrorKind, TResult};
use crate::subtags;
use assyst_common::util::filetype::Type;
use rand::prelude::ThreadRng;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::Range;
use std::string::FromUtf8Error;

/// Constants and helper functions for tag parser limits
pub mod limits {
    use std::cell::Cell;

    pub const MAX_REQUESTS: u32 = 5;
    pub const MAX_VARIABLES: usize = 100;
    pub const MAX_VARIABLE_KEY_LENGTH: usize = 100;
    pub const MAX_VARIABLE_VALUE_LENGTH: usize = MAX_STRING_LENGTH;
    pub const MAX_ITERATIONS: u32 = 500;
    pub const MAX_DEPTH: u32 = 15;
    pub const MAX_STRING_LENGTH: usize = 256_000;

    pub fn try_increment(field_cell: &Cell<u32>, limit: u32) -> bool {
        let field = field_cell.get();
        if field >= limit {
            false
        } else {
            field_cell.set(field + 1);
            true
        }
    }
}

/// Parser state that is shared across multiple parsers
///
/// See comment in `Parser::from_parent` for more details.
#[derive(Clone)]
pub struct SharedState<'a> {
    /// User defined variables
    variables: &'a RefCell<HashMap<String, String>>,
    /// Counter for various limits
    counter: &'a Counter,
    /// The attachment to be responded with, if set
    attachment: &'a RefCell<Option<(Vec<u8>, Type)>>,
}

impl<'a> SharedState<'a> {
    pub fn new(
        variables: &'a RefCell<HashMap<String, String>>,
        counter: &'a Counter,
        attachment: &'a RefCell<Option<(Vec<u8>, Type)>>,
    ) -> Self {
        Self {
            variables,
            counter,
            attachment,
        }
    }

    /// Calls `f` with a mutable reference to the user defined variables
    pub fn with_variables_mut<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut HashMap<String, String>) -> T,
    {
        let mut variables = self.variables.borrow_mut();
        f(&mut variables)
    }

    /// Calls `f` with a reference to the user defined variables
    pub fn with_variables<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&HashMap<String, String>) -> T,
    {
        let variables = self.variables.borrow();
        f(&variables)
    }

    /// Returns a reference to the counter
    pub fn counter(&self) -> &Counter {
        self.counter
    }

    /// Sets the attachment to be responded with
    pub fn set_attachment(&self, buf: Vec<u8>, ty: Type) {
        *self.attachment.borrow_mut() = Some((buf, ty));
    }
}

/// Counter for various limits
#[derive(Default)]
pub struct Counter {
    /// Number of HTTP requests
    requests: Cell<u32>,
    /// Number of parser iterations
    iterations: Cell<u32>,
}

impl Counter {
    /// Tries to increment the requests field if it's not already at the limit
    pub fn try_request(&self) -> bool {
        limits::try_increment(&self.requests, limits::MAX_REQUESTS)
    }

    /// Tries to increment the iterations field if it's not already at the limit
    pub fn try_iterate(&self) -> bool {
        limits::try_increment(&self.iterations, limits::MAX_ITERATIONS)
    }
}

/// The tag parser
pub struct Parser<'a> {
    /// The input string
    input: &'a [u8],
    /// Tag arguments, accessible through {arg} and {args}
    args: &'a [&'a str],
    /// Current index in the input string
    idx: usize,
    /// Shared parser state across multiple parsers
    state: SharedState<'a>,
    /// A cached `ThreadRng`, used to generate random numbers
    rng: ThreadRng,
    /// Context for this parser
    cx: &'a dyn Context,
    /// Recursive depth, to avoid stack overflow in {eval} calls
    depth: u32,
    /// Stack of tag start positions, exclusively used for error reporting.
    tag_start_positions: Vec<BytePos>,
}

/// Checks if a given byte is in the a..z A..Z range
fn is_identifier(b: u8) -> bool {
    b.is_ascii_alphabetic()
}

impl<'a> Parser<'a> {
    /// Creates a parser with shared state from the parent parser
    ///
    /// The returned parser shares the same limits and variables with `other`
    pub fn from_parent(input: &'a [u8], other: &Self) -> Self {
        Self::from_parent_with_args(input, other, other.args)
    }

    /// Creates a parser with shared state from the parent parser.
    /// Allows changing the args for this specific parser
    ///
    /// The returned parser shares the same limits and variables with `other`
    pub fn from_parent_with_args(input: &'a [u8], other: &Self, args: &'a [&'a str]) -> Self {
        Self {
            input,
            args,
            idx: 0,
            state: other.state.clone(),
            rng: rand::thread_rng(),
            cx: other.cx,
            depth: other.depth + 1,
            tag_start_positions: Vec::new(),
        }
    }

    /// Creates a new parser
    pub fn new(input: &'a [u8], args: &'a [&'a str], state: SharedState<'a>, cx: &'a dyn Context) -> Self {
        Self {
            input,
            args,
            cx,
            idx: 0,
            state,
            rng: rand::thread_rng(),
            depth: 0,
            tag_start_positions: Vec::new(),
        }
    }

    /// Reads bytes from input until the first non-identifier byte is found, increasing the internal
    /// index on
    pub fn read_identifier(&mut self) -> &'a [u8] {
        let start = self.idx;

        while self.idx < self.input.len() {
            let b = self.input[self.idx];

            if !is_identifier(b) {
                break;
            }

            self.idx += 1;
        }

        &self.input[start..self.idx]
    }

    pub fn skip_whitespace(&mut self) {
        while self.idx < self.input.len() {
            if !self.input[self.idx].is_ascii_whitespace() {
                break;
            }
            self.idx += 1;
        }
    }

    /// Eats a byte
    pub fn eat(&mut self, bs: &[u8]) -> bool {
        if let Some(b) = self.input.get(self.idx) {
            if bs.contains(b) {
                self.idx += 1;
                return true;
            }
        }
        false
    }

    /// Eats a separator
    pub fn eat_separator(&mut self) -> bool {
        self.eat(b":|")
    }

    /// Checks if the current character is escaped, i.e. the last character is \
    pub fn is_escaped(&self) -> bool {
        self.input.get(self.idx - 1) == Some(&b'\\')
    }

    /// **DO NOT CALL THIS DIRECTLY!**
    /// Always go through `parse_segment`. This is because there is some logic needed when entering
    /// and leaving a segment. Calling this directly will effectively skip it.
    /// In particular, this has no sort of checks for iteration limits.
    /// It is also marked deprecated specifically for that reason.
    #[deprecated = "do not call this"]
    fn parse_segment_inner_untracked(&mut self, side_effects: bool) -> TResult<String> {
        let mut output = Vec::new();

        if !side_effects {
            // If this call isn't allowed to have any side effects, we can just "fast-forward" to the next }
            // that matches the depth of the current call.
            let mut depth = 1;

            while self.idx < self.input.len() {
                let byte = self.input[self.idx];

                match byte {
                    b'{' if !self.is_escaped() => depth += 1,
                    b'}' if !self.is_escaped() => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    },
                    b'|' if !self.is_escaped() && depth <= 1 => break,
                    _ => {},
                }

                output.push(byte);
                self.idx += 1;
            }

            return String::from_utf8(output).map_err(|err| self.unreachable_invalid_utf8(err));
        }

        while self.idx < self.input.len() {
            let byte = self.input[self.idx];

            match byte {
                b'{' => {
                    *self.tag_start_positions.last_mut().unwrap() = self.idx;
                    // skip {
                    self.idx += 1;

                    // skipping whitespace seems sensible and also improves errors
                    self.skip_whitespace();

                    // get subtag name, i.e. `range` in {range:1|10}
                    let (name, name_span) = {
                        let before_idx = self.idx;
                        let name = std::str::from_utf8(self.read_identifier()).unwrap();
                        let after_idx = self.idx;
                        (name, before_idx..after_idx)
                    };

                    // if we're now at the end of the input, tag abruptly stopped without a `}`
                    if self.idx == self.input.len() {
                        return err_res(ErrorKind::MissingClosingBrace {
                            expected_position: self.idx,
                            tag_start: self.last_tag_start_pos(),
                        });
                    }

                    if name.is_empty() {
                        return err_res(ErrorKind::EmptySubtag { span: self.span() });
                    }

                    // lazy tags need to be evaluated before the args are parsed
                    // see comment in `handle_lazy_tag` for what it means for a tag to be lazy
                    if let Some(re) = self.handle_lazy_tag(name) {
                        output.append(&mut re?.into_bytes());
                        continue;
                    }

                    let mut args = Vec::new();

                    while let Some(b'|' | b':') = self.input.get(self.idx) {
                        // skip `|:`
                        self.idx += 1;

                        // recursively parse segment
                        args.push(self.parse_segment(side_effects)?);
                    }

                    // reject code like {eval!}, where the `!` should have been `}`
                    let closing_brace = self.idx;
                    if !self.eat(b"}") {
                        return err_res(ErrorKind::MissingClosingBrace {
                            expected_position: closing_brace,
                            tag_start: self.last_tag_start_pos(),
                        });
                    }

                    let result = if side_effects {
                        self.handle_tag(name, name_span, args)?
                    } else {
                        String::new()
                    };

                    if output.len() + result.len() > limits::MAX_STRING_LENGTH {
                        return err_res(ErrorKind::StringLengthLimit {
                            span: self.span(),
                            attempted_size: output.len() + result.len(),
                        });
                    }

                    output.append(&mut result.into_bytes());
                },
                b'|' | b'}' => {
                    break;
                },
                _ => {
                    // If we are escaping | or }, then only push *that* character, and not \
                    if byte == b'\\' {
                        if let Some(&next @ b'|' | &next @ b'}' | &next @ b'{') = self.input.get(self.idx + 1) {
                            output.push(next);
                            self.idx += 2;
                            continue;
                        }
                    }

                    output.push(byte);
                    self.idx += 1;
                },
            }
        }

        String::from_utf8(output).map_err(|err| self.unreachable_invalid_utf8(err))
    }

    #[cold]
    #[track_caller]
    fn unreachable_invalid_utf8(&self, err: FromUtf8Error) -> ! {
        // this should not be possible -- be as useful as possible for debugging purposes when it does
        // happen
        panic!("tag ended up with invalid utf-8: {err:?}\ntag source: {:?}", self.input)
    }

    /// Parses a single "segment" of the input
    ///
    /// A segment can be a single argument of a tag, or the entire input itself.
    /// Sometimes it's necessary to parse without "side effects", which means
    /// that it needs to parse a segment without actually invoking the tag handler.
    ///
    /// For example, given `a{note:{arg:this_would_error}}b`, if we want to skip the note tag
    /// such that we end up with `ab`, we need to parse it without calling the `arg` tag handler.
    /// If we *did* invoke it, this would return an error
    pub fn parse_segment(&mut self, side_effects: bool) -> TResult<String> {
        if !self.state.counter.try_iterate() {
            return err_res(ErrorKind::IterLimit { pos: self.idx });
        }
        self.tag_start_positions.push(self.idx);
        #[allow(deprecated)]
        let res = self.parse_segment_inner_untracked(side_effects);
        self.tag_start_positions.pop().unwrap();
        res
    }

    /// Handles a "lazy" tag
    ///
    /// Lazy tags are subtags whose arguments are parsed by the subtag itself, and not by the parser
    /// beforehand. This is needed for special subtags like if, which needs to decide whether to
    /// parse `then` or else` only after it compared two arguments
    pub fn handle_lazy_tag(&mut self, name: &str) -> Option<TResult<String>> {
        match name {
            "if" => Some(subtags::r#if(self)),
            "note" => Some(subtags::note(self)),
            "ignore" => Some(subtags::ignore(self)),
            _ => None,
        }
    }

    /// Handles a regular tag
    pub fn handle_tag(&mut self, name: &str, name_span: Range<usize>, args: Vec<String>) -> TResult<String> {
        match name {
            "repeat" => subtags::exec(self, &args, subtags::repeat),
            "range" => subtags::exec(self, &args, subtags::range),
            "eval" => subtags::exec(self, &args, subtags::eval),
            "tryarg" => subtags::exec(self, &args, subtags::tryarg),
            "arg" => subtags::exec(self, &args, subtags::arg),
            "args" => subtags::exec(self, &args, subtags::args),
            "set" => subtags::exec(self, &args, subtags::set),
            "get" => subtags::exec(self, &args, subtags::get),
            "delete" => subtags::exec(self, &args, subtags::delete),
            "argslen" => subtags::exec(self, &args, subtags::argslen),
            "abs" => subtags::exec(self, &args, subtags::abs),
            "cos" => subtags::exec(self, &args, subtags::cos),
            "sin" => subtags::exec(self, &args, subtags::sin),
            "tan" => subtags::exec(self, &args, subtags::tan),
            "sqrt" => subtags::exec(self, &args, subtags::sqrt),
            "e" => subtags::exec(self, &args, subtags::e),
            "pi" => subtags::exec(self, &args, subtags::pi),
            "max" => subtags::exec(self, &args, subtags::max),
            "min" => subtags::exec(self, &args, subtags::min),
            "choose" => subtags::exec(self, &args, subtags::choose),
            "length" => subtags::exec(self, &args, subtags::length),
            "lower" => subtags::exec(self, &args, subtags::lower),
            "upper" => subtags::exec(self, &args, subtags::upper),
            "replace" => subtags::exec(self, &args, subtags::replace),
            "reverse" => subtags::exec(self, &args, subtags::reverse),
            "channelid" => subtags::exec(self, &args, subtags::channelid),
            "usertag" => subtags::exec(self, &args, subtags::usertag),
            "js" | "javascript" => subtags::exec(self, &args, subtags::javascript),
            "lastattachment" => subtags::exec(self, &args, subtags::attachment_last),
            "avatar" => subtags::exec(self, &args, subtags::avatar),
            "download" => subtags::exec(self, &args, subtags::download),
            "mention" => subtags::exec(self, &args, subtags::mention),
            "idof" => subtags::exec(self, &args, subtags::idof),
            "userid" => subtags::exec(self, &args, subtags::userid),
            "tag" => subtags::exec(self, &args, subtags::tag),
            _ => err_res(ErrorKind::UnknownSubtag {
                name: name.to_owned(),
                span: name_span,
            }),
        }
    }

    pub fn args(&self) -> &[&str] {
        self.args
    }

    pub fn rng(&mut self) -> &mut ThreadRng {
        &mut self.rng
    }

    pub fn state(&self) -> &SharedState<'a> {
        &self.state
    }

    pub fn depth(&self) -> u32 {
        self.depth
    }

    pub fn context(&self) -> &'a dyn Context {
        self.cx
    }

    pub fn pos(&self) -> BytePos {
        self.idx
    }

    pub fn eof(&self) -> bool {
        self.idx >= self.input.len()
    }

    /// Only call this within a tag context.
    pub fn last_tag_start_pos(&self) -> BytePos {
        *self.tag_start_positions.last().unwrap()
    }

    /// Only call this within a tag context.
    pub fn span(&self) -> Range<usize> {
        self.last_tag_start_pos()..self.idx
    }

    /// Only call this within a tag context.
    pub fn span_with_hi(&self, hi: BytePos) -> Range<usize> {
        self.last_tag_start_pos()..hi
    }
}
