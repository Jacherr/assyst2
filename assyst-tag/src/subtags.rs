use std::num::{ParseFloatError, ParseIntError};

use crate::errors::{err, err_res, wrap_anyhow, ErrorKind, TResult};
use crate::parser::limits::{
    MAX_DEPTH, MAX_STRING_LENGTH, MAX_VARIABLES, MAX_VARIABLE_KEY_LENGTH, MAX_VARIABLE_VALUE_LENGTH,
};
use crate::parser::Parser;
use assyst_common::eval::FakeEvalImageResponse;
use assyst_common::util::discord::id_from_mention;
use either::Either;
use rand::Rng;

/// Ensures that the HTTP request limit has not been hit yet
///
/// This should be called in tags that issue HTTP requests in any way.
/// Returns with an error if the limit is reached
macro_rules! ensure_request_limit {
    ($parser:expr) => {{
        let parser = &$parser;
        if !parser.state().counter().try_request() {
            return err_res(ErrorKind::RequestLimit { span: parser.span() });
        }
    }};
}

fn try_eat_closing_brace(parser: &mut Parser<'_>) -> TResult<()> {
    if !parser.eat(b"}") {
        err_res(ErrorKind::MissingClosingBrace {
            expected_position: parser.pos(),
            tag_start: parser.last_tag_start_pos(),
        })
    } else {
        Ok(())
    }
}

pub struct ParseSuccess<T> {
    pub value: T,
    pub args_consumed: usize,
}

#[derive(Debug)]
pub enum ParseError {
    MissingArgument,
    // TODO: display required amount
    NotEnoughArguments,
    F64FromStrError(ParseFloatError, String),
    UsizeFromStrError(ParseIntError, String),
    I64FromStrError(ParseIntError, String),
    U64FromStrError(ParseIntError, String),

    Other(String),
}

/// Contiuously parses the rest of the arguments as `T`
pub struct Rest<T: ParseTagArgument>(Vec<T>);

/// Same as `Rest`, but requires at least N elements to be present.
pub struct Atleast<const N: usize, T: ParseTagArgument>(Rest<T>);

pub trait ParseTagArgument: Sized {
    fn parse_from_args(parser: &Parser<'_>, args: &[String]) -> Result<ParseSuccess<Self>, ParseError>;
}

impl<T: ParseTagArgument> ParseTagArgument for Rest<T> {
    fn parse_from_args(parser: &Parser<'_>, args: &[String]) -> Result<ParseSuccess<Self>, ParseError> {
        let mut results = Vec::new();
        let mut args_consumed = 0;
        while args_consumed < args.len() {
            let t = T::parse_from_args(parser, &args[args_consumed..])?;
            args_consumed += t.args_consumed;
            results.push(t.value);
        }
        Ok(ParseSuccess {
            value: Rest(results),
            args_consumed,
        })
    }
}

impl<const N: usize, T: ParseTagArgument> ParseTagArgument for Atleast<N, T> {
    fn parse_from_args(parser: &Parser<'_>, args: &[String]) -> Result<ParseSuccess<Self>, ParseError> {
        let rest = Rest::parse_from_args(parser, args)?;
        if rest.value.0.len() < N {
            return Err(ParseError::MissingArgument);
        }
        Ok(ParseSuccess {
            value: Atleast(rest.value),
            args_consumed: rest.args_consumed,
        })
    }
}

impl ParseTagArgument for () {
    fn parse_from_args(_: &Parser<'_>, _: &[String]) -> Result<ParseSuccess<Self>, ParseError> {
        Ok(ParseSuccess {
            value: (),
            args_consumed: 0,
        })
    }
}

impl<A: ParseTagArgument, B: ParseTagArgument> ParseTagArgument for Either<A, B> {
    fn parse_from_args(parser: &Parser<'_>, args: &[String]) -> Result<ParseSuccess<Self>, ParseError> {
        if let Ok(s) = A::parse_from_args(parser, args) {
            Ok(ParseSuccess {
                value: Either::Left(s.value),
                args_consumed: s.args_consumed,
            })
        } else if let Ok(s) = B::parse_from_args(parser, args) {
            Ok(ParseSuccess {
                value: Either::Right(s.value),
                args_consumed: s.args_consumed,
            })
        } else {
            Err(ParseError::NotEnoughArguments)
        }
    }
}

impl<A: ParseTagArgument> ParseTagArgument for Option<A> {
    fn parse_from_args(parser: &Parser<'_>, args: &[String]) -> Result<ParseSuccess<Self>, ParseError> {
        if let Ok(s) = A::parse_from_args(parser, args) {
            Ok(ParseSuccess {
                value: Some(s.value),
                args_consumed: s.args_consumed,
            })
        } else {
            // TODO: maybe throw an error if kind != MissingArgument ??
            Ok(ParseSuccess {
                value: None,
                args_consumed: 0,
            })
        }
    }
}

impl ParseTagArgument for usize {
    fn parse_from_args(_: &Parser<'_>, args: &[String]) -> Result<ParseSuccess<Self>, ParseError> {
        let [arg, ..] = args else {
            return Err(ParseError::MissingArgument);
        };
        arg.parse()
            .map_err(|err| ParseError::UsizeFromStrError(err, arg.into()))
            .map(|value| ParseSuccess {
                value,
                args_consumed: 1,
            })
    }
}

pub struct Mention(u64);

impl ParseTagArgument for Mention {
    fn parse_from_args(parser: &Parser<'_>, args: &[String]) -> Result<ParseSuccess<Self>, ParseError> {
        let [mention, ..] = args else {
            return Err(ParseError::MissingArgument);
        };
        let id = id_from_mention(mention)
            .or_else(|| parser.context().user_id().ok())
            .ok_or_else(|| ParseError::Other("Invalid mention".to_string()))?;

        Ok(ParseSuccess {
            value: Mention(id),
            args_consumed: 1,
        })
    }
}

impl ParseTagArgument for u64 {
    fn parse_from_args(_: &Parser<'_>, args: &[String]) -> Result<ParseSuccess<Self>, ParseError> {
        let [arg, ..] = args else {
            return Err(ParseError::MissingArgument);
        };
        arg.parse()
            .map_err(|err| ParseError::U64FromStrError(err, arg.into()))
            .map(|value| ParseSuccess {
                value,
                args_consumed: 1,
            })
    }
}

impl ParseTagArgument for f64 {
    fn parse_from_args(_: &Parser<'_>, args: &[String]) -> Result<ParseSuccess<Self>, ParseError> {
        let [arg, ..] = args else {
            return Err(ParseError::MissingArgument);
        };
        arg.parse()
            .map_err(|err| ParseError::F64FromStrError(err, arg.into()))
            .map(|value| ParseSuccess {
                value,
                args_consumed: 1,
            })
    }
}

impl ParseTagArgument for i64 {
    fn parse_from_args(_: &Parser<'_>, args: &[String]) -> Result<ParseSuccess<Self>, ParseError> {
        let [arg, ..] = args else {
            return Err(ParseError::MissingArgument);
        };
        arg.parse()
            .map_err(|err| ParseError::I64FromStrError(err, arg.into()))
            .map(|value| ParseSuccess {
                value,
                args_consumed: 1,
            })
    }
}

impl ParseTagArgument for String {
    fn parse_from_args(_: &Parser<'_>, args: &[String]) -> Result<ParseSuccess<Self>, ParseError> {
        let [arg, ..] = args else {
            return Err(ParseError::MissingArgument);
        };
        Ok(ParseSuccess {
            value: arg.clone(),
            args_consumed: 1,
        })
    }
}

impl<A, B> ParseTagArgument for (A, B)
where
    A: ParseTagArgument,
    B: ParseTagArgument,
{
    fn parse_from_args(parser: &Parser<'_>, args: &[String]) -> Result<ParseSuccess<Self>, ParseError> {
        let ParseSuccess {
            value: a,
            args_consumed: a_consumed,
        } = A::parse_from_args(parser, args)?;
        let ParseSuccess {
            value: b,
            args_consumed: b_consumed,
        } = B::parse_from_args(parser, &args[a_consumed..])?;
        Ok(ParseSuccess {
            value: (a, b),
            args_consumed: a_consumed + b_consumed,
        })
    }
}

pub trait Subtag {
    fn exec(&self, parser: &mut Parser<'_>, args: &[String]) -> TResult<String>;
}

impl<A: ParseTagArgument> Subtag for fn(&mut Parser<'_>, A) -> TResult<String> {
    fn exec(&self, parser: &mut Parser<'_>, args: &[String]) -> TResult<String> {
        let ParseSuccess { value, .. } = A::parse_from_args(parser, args).map_err(|er| {
            err(ErrorKind::ArgParseError {
                err: er,
                span: parser.span(),
            })
        })?;
        self(parser, value)
    }
}

/// Convenience wrapper function that doesn't require ugly casts
pub fn exec<A: ParseTagArgument>(
    p: &mut Parser<'_>,
    args: &[String],
    f: fn(&mut Parser<'_>, A) -> TResult<String>,
) -> TResult<String> {
    Subtag::exec(&f, p, args)
}

pub fn repeat(parser: &mut Parser<'_>, (count, input): (usize, String)) -> TResult<String> {
    if input.len() + count > MAX_STRING_LENGTH {
        return err_res(ErrorKind::StringLengthLimit {
            span: parser.span(),
            attempted_size: input.len() + count,
        });
    }

    Ok(input.repeat(count))
}

pub fn range(parser: &mut Parser<'_>, (lower, upper): (usize, usize)) -> TResult<String> {
    let out: usize = parser.rng().gen_range(lower..=upper);

    Ok(out.to_string())
}

pub fn eval(parser: &mut Parser<'_>, text: String) -> TResult<String> {
    if parser.depth() >= MAX_DEPTH {
        return err_res(ErrorKind::DepthLimit { span: parser.span() });
    }
    crate::parse_with_parent(&text, parser, true).map_err(|error| err(ErrorKind::Nested { source: text, error }))
}

pub fn arg(parser: &mut Parser<'_>, idx: usize) -> TResult<String> {
    parser.args().get(idx).map(|v| v.to_string()).ok_or_else(|| {
        err(ErrorKind::IndexOutOfBounds {
            used_idx: idx,
            len: parser.args().len(),
            span: parser.span(),
        })
    })
}

pub fn tryarg(parser: &mut Parser<'_>, idx: usize) -> TResult<String> {
    Ok(parser.args().get(idx).map(|v| v.to_string()).unwrap_or_default())
}

pub fn args(parser: &mut Parser<'_>, _: ()) -> TResult<String> {
    Ok(parser.args().join(" "))
}

pub fn set(parser: &mut Parser<'_>, (key, value): (String, String)) -> TResult<String> {
    parser.state().with_variables_mut(|vars| -> TResult<String> {
        if vars.len() >= MAX_VARIABLES {
            return err_res(ErrorKind::VarLimit { span: parser.span() });
        }

        if key.len() > MAX_VARIABLE_KEY_LENGTH {
            return err_res(ErrorKind::VarKeyLengthLimit {
                span: parser.span(),
                length: key.len(),
            });
        }

        if value.len() > MAX_VARIABLE_VALUE_LENGTH {
            return err_res(ErrorKind::VarValueLengthLimit {
                span: parser.span(),
                length: value.len(),
            });
        }

        vars.insert(key, value);

        Ok(String::new())
    })
}

pub fn get(parser: &mut Parser<'_>, key: String) -> TResult<String> {
    parser
        .state()
        .with_variables(|vars| -> TResult<String> { Ok(vars.get(&key).map(Clone::clone).unwrap_or_default()) })
}

pub fn delete(parser: &mut Parser<'_>, key: String) -> TResult<String> {
    parser.state().with_variables_mut(|vars| -> TResult<String> {
        vars.remove(&key);
        Ok(String::new())
    })
}

pub fn argslen(parser: &mut Parser<'_>, _: ()) -> TResult<String> {
    Ok(parser.args().len().to_string())
}

pub fn abs(_: &mut Parser<'_>, arg: i64) -> TResult<String> {
    Ok(arg.abs().to_string())
}

pub fn cos(_: &mut Parser<'_>, arg: f64) -> TResult<String> {
    Ok(arg.cos().to_string())
}

pub fn sin(_: &mut Parser<'_>, arg: f64) -> TResult<String> {
    Ok(arg.sin().to_string())
}

pub fn tan(_: &mut Parser<'_>, arg: f64) -> TResult<String> {
    Ok(arg.tan().to_string())
}

pub fn sqrt(_: &mut Parser<'_>, arg: f64) -> TResult<String> {
    Ok(arg.sqrt().to_string())
}

pub fn e(_: &mut Parser<'_>, _: ()) -> TResult<String> {
    Ok(std::f64::EPSILON.to_string())
}

pub fn pi(_: &mut Parser<'_>, _: ()) -> TResult<String> {
    Ok(std::f64::consts::PI.to_string())
}

pub fn max(_: &mut Parser<'_>, (initial, Rest(args)): (i64, Rest<i64>)) -> TResult<String> {
    Ok(args.iter().fold(initial, |p, &c| p.max(c)).to_string())
}

pub fn min(_: &mut Parser<'_>, (initial, Rest(args)): (i64, Rest<i64>)) -> TResult<String> {
    Ok(args.iter().fold(initial, |p, &c| p.min(c)).to_string())
}

pub fn choose(parser: &mut Parser<'_>, Atleast(Rest(args)): Atleast<1, String>) -> TResult<String> {
    let idx = parser.rng().gen_range(0..args.len());
    Ok(args.get(idx).cloned().expect("0..len should always be inbounds"))
}

pub fn length(_: &mut Parser<'_>, arg: String) -> TResult<String> {
    Ok(arg.len().to_string())
}

pub fn lower(_: &mut Parser<'_>, mut arg: String) -> TResult<String> {
    arg.make_ascii_lowercase();
    Ok(arg)
}

pub fn upper(_: &mut Parser<'_>, mut arg: String) -> TResult<String> {
    arg.make_ascii_uppercase();
    Ok(arg)
}

pub fn replace(_: &mut Parser<'_>, (what, (with, text)): (String, (String, String))) -> TResult<String> {
    Ok(text.replace(&what, &with))
}

pub fn reverse(_: &mut Parser<'_>, text: String) -> TResult<String> {
    let mut bytes = text.into_bytes();
    bytes.reverse();
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

pub fn r#if(parser: &mut Parser<'_>) -> TResult<String> {
    if !parser.eat_separator() {
        return err_res(ErrorKind::IfMissingStmt { span: parser.span() });
    }
    let mut stmt = parser.parse_segment(true)?;

    if !parser.eat_separator() {
        return err_res(ErrorKind::IfMissingCmp { span: parser.span() });
    }
    let comparison = parser.parse_segment(true)?;

    if !parser.eat_separator() {
        return err_res(ErrorKind::IfMissingValue { span: parser.span() });
    }
    let mut value = parser.parse_segment(true)?;

    if !parser.eat_separator() {
        return err_res(ErrorKind::IfMissingThen { span: parser.span() });
    }

    fn eval_branch(parser: &mut Parser<'_>, condition: bool) -> TResult<String> {
        if condition {
            let then = parser.parse_segment(true)?;
            if !parser.eat_separator() {
                return err_res(ErrorKind::IfMissingElse { span: parser.span() });
            }
            parser.parse_segment(false)?;
            Ok(then)
        } else {
            parser.parse_segment(false)?;
            if !parser.eat_separator() {
                return err_res(ErrorKind::IfMissingElse { span: parser.span() });
            }
            parser.parse_segment(true)
        }
    }
    fn eval_branch_with_i32s<F>(parser: &mut Parser<'_>, a: &str, b: &str, f: F) -> TResult<String>
    where
        F: FnOnce(i32, i32) -> bool,
    {
        let (a, b) = (
            a.parse().map_err(|error| {
                err(ErrorKind::ArgParseError {
                    span: parser.span(),
                    err: ParseError::I64FromStrError(error, a.to_owned()),
                })
            })?,
            b.parse().map_err(|error| {
                err(ErrorKind::ArgParseError {
                    span: parser.span(),
                    err: ParseError::I64FromStrError(error, b.to_owned()),
                })
            })?,
        );
        eval_branch(parser, f(a, b))
    }

    let result = match comparison.as_str() {
        "=" => eval_branch(parser, stmt == value),
        ">" => eval_branch_with_i32s(parser, &stmt, &value, |a, b| a > b),
        ">=" => eval_branch_with_i32s(parser, &stmt, &value, |a, b| a >= b),
        "<" => eval_branch_with_i32s(parser, &stmt, &value, |a, b| a < b),
        "<=" => eval_branch_with_i32s(parser, &stmt, &value, |a, b| a <= b),
        "~" => {
            stmt.make_ascii_lowercase();
            value.make_ascii_lowercase();
            eval_branch(parser, stmt == value)
        },
        _ => err_res(ErrorKind::IfInvalidCmp { span: parser.span() }),
    };

    try_eat_closing_brace(parser)?;
    result
}

pub fn note(parser: &mut Parser<'_>) -> TResult<String> {
    if parser.eat_separator() {
        parser.parse_segment(false)?;
    }

    try_eat_closing_brace(parser)?;
    Ok(String::new())
}

pub fn ignore(parser: &mut Parser<'_>) -> TResult<String> {
    let result = if parser.eat_separator() {
        parser.parse_segment(false)?
    } else {
        String::new()
    };

    try_eat_closing_brace(parser)?;
    Ok(result)
}

pub fn mention(parser: &mut Parser<'_>, id: Option<u64>) -> TResult<String> {
    Ok(format!(
        "<@{}>",
        match id {
            Some(id) => id,
            None => parser
                .context()
                .user_id()
                .map_err(|err| wrap_anyhow(parser.span(), err))?,
        }
    ))
}

pub fn usertag(parser: &mut Parser<'_>, id: Option<u64>) -> TResult<String> {
    ensure_request_limit!(parser);

    parser
        .context()
        .user_tag(id)
        .map_err(|err| wrap_anyhow(parser.span(), err))
}

pub fn javascript(parser: &mut Parser<'_>, code: String) -> TResult<String> {
    ensure_request_limit!(parser);

    let result = parser
        .context()
        .execute_javascript(&code, parser.args().iter().map(|x| x.to_string()).collect::<Vec<_>>())
        .map_err(|err| wrap_anyhow(parser.span(), err))?;

    match result {
        FakeEvalImageResponse::Image(img, ty) => {
            parser.state().set_attachment(img, ty);
            Ok(String::new())
        },
        FakeEvalImageResponse::Text(t) => Ok(t.message),
    }
}

pub fn attachment_last(parser: &mut Parser<'_>, _: ()) -> TResult<String> {
    ensure_request_limit!(parser);

    parser
        .context()
        .get_last_attachment()
        .map_err(|err| wrap_anyhow(parser.span(), err))
}

pub fn avatar(parser: &mut Parser<'_>, id: Option<u64>) -> TResult<String> {
    ensure_request_limit!(parser);

    parser
        .context()
        .get_avatar(id)
        .map_err(|err| wrap_anyhow(parser.span(), err))
}

pub fn download(parser: &mut Parser<'_>, url: String) -> TResult<String> {
    ensure_request_limit!(parser);

    parser
        .context()
        .download(url.trim())
        .map_err(|err| wrap_anyhow(parser.span(), err))
}

pub fn channelid(parser: &mut Parser<'_>, _: ()) -> TResult<String> {
    Ok(parser
        .context()
        .channel_id()
        .map_err(|err| wrap_anyhow(parser.span(), err))?
        .to_string())
}

pub fn userid(parser: &mut Parser<'_>, _: ()) -> TResult<String> {
    Ok(parser
        .context()
        .user_id()
        .map_err(|err| wrap_anyhow(parser.span(), err))?
        .to_string())
}

pub fn idof(_: &mut Parser<'_>, mention: Mention) -> TResult<String> {
    Ok(mention.0.to_string())
}

pub fn tag(parser: &mut Parser<'_>, (name, Rest(args)): (String, Rest<String>)) -> TResult<String> {
    if parser.depth() >= MAX_DEPTH {
        return err_res(ErrorKind::DepthLimit { span: parser.span() });
    }
    let content = parser
        .context()
        .get_tag_contents(&name)
        .map_err(|err| wrap_anyhow(parser.span(), err))?;

    let args = args.iter().map(|s| s.as_str()).collect::<Vec<_>>();

    Parser::from_parent_with_args(content.as_bytes(), parser, &args).parse_segment(true)
}
