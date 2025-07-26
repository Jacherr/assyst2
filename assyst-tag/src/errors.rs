use std::borrow::Cow;
use std::fmt::Arguments;
use std::ops::Range;

use assyst_string_fmt::Ansi;
use memchr::memmem::rfind;

use crate::parser::limits;
use crate::subtags;
use crate::subtags::ParseError;

pub type BytePos = usize;

pub fn err(kind: ErrorKind) -> Error {
    Error { kind: Box::new(kind) }
}
pub fn err_res<T>(kind: ErrorKind) -> TResult<T> {
    Err(err(kind))
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    /// Iteration limit exceeded
    IterLimit {
        /// Position at which the limit was exceeded
        pos: BytePos,
    },
    EmptySubtag {
        span: Range<usize>,
    },
    UnknownSubtag {
        name: String,
        span: Range<usize>,
    },
    DepthLimit {
        span: Range<usize>,
    },
    VarLimit {
        span: Range<usize>,
    },
    VarKeyLengthLimit {
        length: usize,
        span: Range<usize>,
    },
    VarValueLengthLimit {
        length: usize,
        span: Range<usize>,
    },
    StringLengthLimit {
        span: Range<usize>,
        attempted_size: usize,
    },
    RequestLimit {
        span: Range<usize>,
    },
    ArgParseError {
        span: Range<usize>,
        err: subtags::ParseError,
    },
    IndexOutOfBounds {
        used_idx: usize,
        len: usize,
        span: Range<usize>,
    },

    /// Missing statement in {if} tag
    IfMissingStmt {
        span: Range<usize>,
    },
    /// Missing comparison in {if} tag
    IfMissingCmp {
        span: Range<usize>,
    },
    /// Missing value in {if} tag
    IfMissingValue {
        span: Range<usize>,
    },
    /// Missing "then" in {if} tag
    IfMissingThen {
        span: Range<usize>,
    },
    /// Missing "else" in {if} tag
    IfMissingElse {
        span: Range<usize>,
    },
    /// Invalid comparison in {if} tag
    IfInvalidCmp {
        span: Range<usize>,
    },

    MissingClosingBrace {
        expected_position: BytePos,
        tag_start: BytePos,
    },

    /// A nested error, possibly created by subparsers
    Nested {
        source: String,
        error: Error,
    },

    Unknown {
        span: Range<usize>,
        message: String,
    },
}

#[derive(Debug, Clone)]
pub struct Error {
    // inner error is boxed because we want to keep the size of `Result<_, Error>` small with
    // respect to the common case that is `Ok`
    pub kind: Box<ErrorKind>,
}

pub type TResult<T> = Result<T, Error>;

pub fn wrap_anyhow(at: Range<usize>, res: anyhow::Error) -> Error {
    err(ErrorKind::Unknown {
        span: at,
        message: res.to_string(),
    })
}

struct LineData<'a> {
    relative_span_lo: usize,
    relative_span_hi: usize,
    line: &'a str,
}

fn line_data(source: &str, span: Range<usize>) -> LineData<'_> {
    let start_index = rfind(&source.as_bytes()[..span.start.min(source.len())], b"\n")
        .map(|x| x + 1)
        .unwrap_or(0);

    let end_index = if span.end >= source.len() {
        // Allow pointing at one past the input buffer
        source.len()
    } else {
        memchr::memchr(b'\n', &source.as_bytes()[span.end..])
            .map(|x| x + span.end)
            .unwrap_or(source.len())
    };

    let relative_span_lo = span.start - start_index;
    let relative_span_hi = span.end - start_index;

    let line = &source[start_index..end_index.min(source.len())];
    LineData {
        relative_span_lo,
        relative_span_hi,
        line,
    }
}

pub enum DiagnosticKind {
    Error,
    Warning,
}

pub enum NoteKind {
    Error,
    Warning,
    Note,
    Help,
}

pub struct Note {
    kind: NoteKind,
    span: Option<Range<usize>>,
    message: Cow<'static, str>,
}

pub struct DiagnosticBuilder<'buf> {
    src: &'buf str,
    kind: DiagnosticKind,
    message: Option<Cow<'static, str>>,
    span_notes: Vec<Note>,
}

impl<'buf> DiagnosticBuilder<'buf> {
    pub fn into_string(self) -> String {
        let mut out = String::new();

        match self.kind {
            DiagnosticKind::Error => out.push_str(&"error: ".fg_red().a_bold()),
            DiagnosticKind::Warning => out.push_str(&"warning: ".fg_yellow().a_bold()),
        }

        out += &self.message.expect("no message set for diagnostic");
        out += "\n\n";

        for (index, Note { kind, span, message }) in self.span_notes.iter().enumerate() {
            if index > 0 {
                out += "\n\n";
            }

            match span.clone() {
                Some(span) => {
                    let LineData {
                        relative_span_lo,
                        relative_span_hi,
                        line,
                    } = line_data(self.src, span);

                    out += &" | ".fg_blue();
                    out += line;
                    out += "\n";

                    out += &" ".repeat(3 + relative_span_lo);

                    let arrows = "^".repeat(relative_span_hi - relative_span_lo);
                    match kind {
                        NoteKind::Error => {
                            out += &arrows.fg_red().a_bold();
                            out += " ";
                            out += &message.fg_red().a_bold();
                        },
                        NoteKind::Warning => {
                            out += &arrows.fg_yellow().a_bold();
                            out += " ";
                            out += &message.fg_yellow().a_bold();
                        },
                        NoteKind::Help => {
                            out += &arrows.fg_cyan().a_bold();
                            out += " ";
                            out += &message.fg_cyan().a_bold();
                        },
                        NoteKind::Note => {
                            out += &arrows.a_bold();
                            out += " ";
                            out += message;
                        },
                    }
                },
                None => {
                    match kind {
                        NoteKind::Error => {
                            out += &"error: ".a_bold();
                        },
                        NoteKind::Warning => {
                            out += &"warning: ".a_bold();
                        },
                        NoteKind::Help => {
                            out += &"help: ".fg_cyan().a_bold();
                        },
                        NoteKind::Note => {
                            out += &"note: ".a_bold();
                        },
                    }
                    out += message;
                },
            }
        }

        out
    }
}

pub fn format_error(src: &str, err: Error) -> String {
    fn char_index_to_span(src: &str, index: usize) -> Range<usize> {
        let lo = src.floor_char_boundary(index);
        if let Some(c) = src[lo..].chars().next() {
            lo..lo + c.len_utf8()
        } else {
            // Out of bounds -- create a span that is one byte wide
            lo..lo + 1
        }
    }

    if let ErrorKind::Nested { source, error } = *err.kind {
        return format_error(&source, error);
    }

    let mut db = DiagnosticBuilder {
        src,
        kind: DiagnosticKind::Error,
        message: None,
        span_notes: Vec::new(),
    };

    fn simple_span_diag(db: &mut DiagnosticBuilder<'_>, args: Arguments<'_>, span: Option<Range<usize>>) {
        db.message = Some(match args.as_str() {
            Some(msg) => Cow::Borrowed(msg),
            None => Cow::Owned(args.to_string()),
        });
        if let Some(span) = span {
            db.span_notes.push(Note {
                kind: NoteKind::Error,
                message: "".into(),
                span: Some(span),
            });
        }
    }

    match *err.kind {
        ErrorKind::EmptySubtag { span } => {
            db.message = Some("subtag name is empty".into());
            db.span_notes.push(Note {
                kind: NoteKind::Error,
                message: "tag starts here, but it does not have a name".into(),
                span: Some(span),
            });
            db.span_notes.push(Note {
                kind: NoteKind::Help,
                message: "try wrapping it in `{ignore:...}` to avoid interpretation as a subtag".into(),
                span: None,
            });
            db.span_notes.push(Note {
                kind: NoteKind::Help,
                message: "... or try escaping the `{` and `}` characters with `\\{...\\}`".into(),
                span: None,
            });
        },
        ErrorKind::StringLengthLimit { span, attempted_size } => {
            db.message = Some("maximum output string length exceeded".into());
            db.span_notes.push(Note {
                kind: NoteKind::Error,
                message: format!("string being created is {attempted_size} bytes").into(),
                span: Some(span),
            });
        },
        //===untested====
        ErrorKind::DepthLimit { span } => {
            db.message = Some("maximum recursion depth exceeded".into());
            db.span_notes.push(Note {
                kind: NoteKind::Error,
                message: "error occurred at this node".into(),
                span: Some(span),
            });
        },
        ErrorKind::IndexOutOfBounds { span, used_idx, len } => {
            db.message = Some("argument index is out of bounds".into());
            db.span_notes.push(Note {
                kind: NoteKind::Error,
                message: format!("the index is {used_idx} but the length is {len}").into(),
                span: Some(span),
            });
            db.span_notes.push(Note {
                kind: NoteKind::Help,
                message: "consider using {tryarg:...} to get an empty string on out-of-bounds access".into(),
                span: None,
            });
            db.span_notes.push(Note {
                kind: NoteKind::Help,
                message: "... or handle the out-of-bounds case explicitly with {if:{argslen}|>|index|{arg:index}|...}"
                    .into(),
                span: None,
            });
        },
        ErrorKind::ArgParseError {
            span,
            err:
                subtags::ParseError::U64FromStrError(er, input)
                | subtags::ParseError::I64FromStrError(er, input)
                | subtags::ParseError::UsizeFromStrError(er, input),
        } => {
            db.message = Some(format!("failed to parse '{input}' as a number").into());
            db.kind = DiagnosticKind::Error;
            db.span_notes.push(Note {
                kind: NoteKind::Error,
                message: format!("{er}").into(),
                span: Some(span),
            });
            // TODO: be more elaborate and show a help... "this arg was expected to be a number"
        },
        ErrorKind::ArgParseError {
            span,
            err: subtags::ParseError::F64FromStrError(err, input),
        } => {
            db.message = Some(format!("failed to parse '{input}' as a float").into());
            db.kind = DiagnosticKind::Error;
            db.span_notes.push(Note {
                kind: NoteKind::Error,
                message: err.to_string().into(),
                // TODO: use the span of the exact argument
                span: Some(span),
            });
        },
        ErrorKind::ArgParseError {
            span,
            err: ParseError::MissingArgument,
        } => simple_span_diag(&mut db, format_args!("another argument is expected"), Some(span)),

        ErrorKind::ArgParseError {
            span,
            err: ParseError::NotEnoughArguments,
        } => simple_span_diag(&mut db, format_args!("more arguments are expected"), Some(span)),
        ErrorKind::ArgParseError {
            span,
            err: ParseError::Other(other),
        } => simple_span_diag(&mut db, format_args!("{other}"), Some(span)),
        ErrorKind::IterLimit { pos } => {
            db.message = Some("tag iteration limit exceeded".into());
            db.span_notes.push(Note {
                kind: NoteKind::Error,
                message: "ran out of iteration time while processing this token".into(),
                span: Some(char_index_to_span(src, pos)),
            });
        },
        ErrorKind::MissingClosingBrace {
            expected_position,
            tag_start,
        } => {
            db.message = Some("missing closing brace".into());
            db.span_notes.push(Note {
                kind: NoteKind::Error,
                message: "expected a closing brace here".into(),
                span: Some(char_index_to_span(src, expected_position)),
            });
            db.span_notes.push(Note {
                kind: NoteKind::Help,
                message: "tag parsing begins here".into(),
                // does not need boundary flooring/ceiling normalization because `}` is always 1
                // byte
                span: Some(tag_start..tag_start + 1),
            });
        },
        ErrorKind::UnknownSubtag { name, span } => {
            db.message = Some(format!("subtag '{name}' does not exist").into());
            db.span_notes.push(Note {
                kind: NoteKind::Error,
                message: "".into(),
                span: Some(span),
            });
            db.span_notes.push(Note {
                kind: NoteKind::Note,
                message: "see https://jacher.io/tags for an exhaustive list of valid subtags".into(),
                span: None,
            });
        },
        ErrorKind::VarLimit { span } => simple_span_diag(
            &mut db,
            format_args!("cannot define more than {} variables", limits::MAX_VARIABLES),
            Some(span),
        ),
        ErrorKind::VarKeyLengthLimit { span, length } => simple_span_diag(
            &mut db,
            format_args!(
                "variable name has too many characters ({}>{})",
                length,
                limits::MAX_VARIABLE_KEY_LENGTH,
            ),
            Some(span),
        ),
        ErrorKind::VarValueLengthLimit { span, length } => simple_span_diag(
            &mut db,
            format_args!(
                "variable value is too long ({}>{})",
                length,
                limits::MAX_VARIABLE_VALUE_LENGTH,
            ),
            Some(span),
        ),
        ErrorKind::RequestLimit { span } => simple_span_diag(
            &mut db,
            format_args!("maximum number of http requests ({}) reached", limits::MAX_REQUESTS),
            Some(span),
        ),
        ErrorKind::IfMissingStmt { span } => simple_span_diag(
            &mut db,
            format_args!("`if` tag is missing a value to compare"),
            Some(span),
        ),
        ErrorKind::IfMissingCmp { span } => {
            simple_span_diag(&mut db, format_args!("`if` tag is missing a comparator"), Some(span))
        },
        ErrorKind::IfMissingValue { span } => simple_span_diag(
            &mut db,
            format_args!("`if` tag is missing a condition that the value is compared to"),
            Some(span),
        ),
        ErrorKind::IfMissingThen { span } => {
            simple_span_diag(&mut db, format_args!("`if` tag is missing a 'then' branch"), Some(span))
        },
        ErrorKind::IfMissingElse { span } => simple_span_diag(
            &mut db,
            format_args!("`if` tag is missing an 'else' branch"),
            Some(span),
        ),
        ErrorKind::IfInvalidCmp { span } => {
            simple_span_diag(&mut db, format_args!("an invalid comparator was used"), Some(span))
        },
        ErrorKind::Nested { .. } => unreachable!("nested tag errors are handled separately"),
        ErrorKind::Unknown { message, span } => {
            db.message = Some(message.into());
            db.span_notes.push(Note {
                kind: NoteKind::Error,
                message: "".into(),
                span: Some(span),
            })
        },
    }

    db.into_string()
}
