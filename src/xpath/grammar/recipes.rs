use std::ops::RangeFrom;

use nom::{
    error::{ErrorKind, ParseError, VerboseError},
    AsChar, IResult, InputIter, Slice,
};

pub type Res<T, U> = IResult<T, U, VerboseError<T>>;

pub fn char_if<I, Error: ParseError<I>>(
    f: fn(char) -> bool,
) -> impl Fn(I) -> IResult<I, char, Error>
where
    I: Slice<RangeFrom<usize>> + InputIter,
    <I as InputIter>::Item: AsChar,
{
    move |i: I| match (i).iter_elements().next().map(|t| {
        let c = t.as_char();
        let b = (f)(c);
        (c, b)
    }) {
        Some((c, true)) => Ok((i.slice(c.len()..), c.as_char())),
        _ => Err(nom::Err::Error(Error::from_error_kind(i, ErrorKind::Char))),
    }
}

pub fn alphabetic<I, Error: ParseError<I>>() -> impl Fn(I) -> IResult<I, char, Error>
where
    I: Slice<RangeFrom<usize>> + InputIter,
    <I as InputIter>::Item: AsChar,
{
    char_if(|c| c.is_ascii_alphabetic())
}

pub fn numeric<I, Error: ParseError<I>>() -> impl Fn(I) -> IResult<I, char, Error>
where
    I: Slice<RangeFrom<usize>> + InputIter,
    <I as InputIter>::Item: AsChar,
{
    char_if(|c| c.is_numeric())
}

pub fn not_quote<I, Error: ParseError<I>>() -> impl Fn(I) -> IResult<I, char, Error>
where
    I: Slice<RangeFrom<usize>> + InputIter,
    <I as InputIter>::Item: AsChar,
{
    char_if(|c| c != '"')
}

pub fn not_single_quote<I, Error: ParseError<I>>() -> impl Fn(I) -> IResult<I, char, Error>
where
    I: Slice<RangeFrom<usize>> + InputIter,
    <I as InputIter>::Item: AsChar,
{
    char_if(|c| c != '\'')
}

pub fn not_brace<I, Error: ParseError<I>>() -> impl Fn(I) -> IResult<I, char, Error>
where
    I: Slice<RangeFrom<usize>> + InputIter,
    <I as InputIter>::Item: AsChar,
{
    char_if(|c| c != '{' && c != '}')
}
