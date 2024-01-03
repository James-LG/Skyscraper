use nom::{
    character::complete::multispace0,
    combinator::opt,
    error::{ParseError, VerboseError},
    Err as NomErr, IResult, Offset, Parser,
};

use super::terminal_symbols::symbol_separator;

/// Parses a list of whitespace-separated items.
///
/// Allows leading whitespace, but does not consume trailing whitespace.
pub fn ws<I: Clone, O, E: ParseError<I>, List: Whitespace<I, O, E>>(
    mut l: List,
) -> impl FnMut(I) -> IResult<I, O, E> {
    move |i: I| l.parse(i)
}

pub trait Whitespace<I, O, E> {
    /// Tests each parser with a symbol separator between them.
    fn parse(&mut self, input: I) -> IResult<I, O, E>;
}

fn ws_parse<'a, Output, A: Parser<&'a str, Output, VerboseError<&'a str>>>(
    input: &'a str,
    parser: &mut A,
) -> Result<(&'a str, Output), NomErr<VerboseError<&'a str>>> {
    let (next_input, _) = multispace0(input)?;

    parser.parse(next_input)
}

impl<'a, Output1, A: Parser<&'a str, Output1, VerboseError<&'a str>>>
    Whitespace<&'a str, Output1, VerboseError<&'a str>> for (A,)
{
    fn parse(&mut self, input: &'a str) -> IResult<&'a str, Output1, VerboseError<&'a str>> {
        ws_parse(input, &mut self.0)
    }
}

impl<
        'a,
        Output1,
        Output2,
        A: Parser<&'a str, Output1, VerboseError<&'a str>>,
        B: Parser<&'a str, Output2, VerboseError<&'a str>>,
    > Whitespace<&'a str, (Output1, Output2), VerboseError<&'a str>> for (A, B)
{
    fn parse(
        &mut self,
        input: &'a str,
    ) -> IResult<&'a str, (Output1, Output2), VerboseError<&'a str>> {
        let (input, res1) = ws_parse(input, &mut self.0)?;
        let (input, res2) = ws_parse(input, &mut self.1)?;
        Ok((input, (res1, res2)))
    }
}

impl<
        'a,
        Output1,
        Output2,
        Output3,
        A: Parser<&'a str, Output1, VerboseError<&'a str>>,
        B: Parser<&'a str, Output2, VerboseError<&'a str>>,
        C: Parser<&'a str, Output3, VerboseError<&'a str>>,
    > Whitespace<&'a str, (Output1, Output2, Output3), VerboseError<&'a str>> for (A, B, C)
{
    fn parse(
        &mut self,
        input: &'a str,
    ) -> IResult<&'a str, (Output1, Output2, Output3), VerboseError<&'a str>> {
        let (input, res1) = ws_parse(input, &mut self.0)?;
        let (input, res2) = ws_parse(input, &mut self.1)?;
        let (input, res3) = ws_parse(input, &mut self.2)?;
        Ok((input, (res1, res2, res3)))
    }
}

impl<
        'a,
        Output1,
        Output2,
        Output3,
        Output4,
        A: Parser<&'a str, Output1, VerboseError<&'a str>>,
        B: Parser<&'a str, Output2, VerboseError<&'a str>>,
        C: Parser<&'a str, Output3, VerboseError<&'a str>>,
        D: Parser<&'a str, Output4, VerboseError<&'a str>>,
    > Whitespace<&'a str, (Output1, Output2, Output3, Output4), VerboseError<&'a str>>
    for (A, B, C, D)
{
    fn parse(
        &mut self,
        input: &'a str,
    ) -> IResult<&'a str, (Output1, Output2, Output3, Output4), VerboseError<&'a str>> {
        let (input, res1) = ws_parse(input, &mut self.0)?;
        let (input, res2) = ws_parse(input, &mut self.1)?;
        let (input, res3) = ws_parse(input, &mut self.2)?;
        let (input, res4) = ws_parse(input, &mut self.3)?;
        Ok((input, (res1, res2, res3, res4)))
    }
}

impl<
        'a,
        Output1,
        Output2,
        Output3,
        Output4,
        Output5,
        A: Parser<&'a str, Output1, VerboseError<&'a str>>,
        B: Parser<&'a str, Output2, VerboseError<&'a str>>,
        C: Parser<&'a str, Output3, VerboseError<&'a str>>,
        D: Parser<&'a str, Output4, VerboseError<&'a str>>,
        E: Parser<&'a str, Output5, VerboseError<&'a str>>,
    > Whitespace<&'a str, (Output1, Output2, Output3, Output4, Output5), VerboseError<&'a str>>
    for (A, B, C, D, E)
{
    fn parse(
        &mut self,
        input: &'a str,
    ) -> IResult<&'a str, (Output1, Output2, Output3, Output4, Output5), VerboseError<&'a str>>
    {
        let (input, res1) = ws_parse(input, &mut self.0)?;
        let (input, res2) = ws_parse(input, &mut self.1)?;
        let (input, res3) = ws_parse(input, &mut self.2)?;
        let (input, res4) = ws_parse(input, &mut self.3)?;
        let (input, res5) = ws_parse(input, &mut self.4)?;
        Ok((input, (res1, res2, res3, res4, res5)))
    }
}

impl<
        'a,
        Output1,
        Output2,
        Output3,
        Output4,
        Output5,
        Output6,
        A: Parser<&'a str, Output1, VerboseError<&'a str>>,
        B: Parser<&'a str, Output2, VerboseError<&'a str>>,
        C: Parser<&'a str, Output3, VerboseError<&'a str>>,
        D: Parser<&'a str, Output4, VerboseError<&'a str>>,
        E: Parser<&'a str, Output5, VerboseError<&'a str>>,
        F: Parser<&'a str, Output6, VerboseError<&'a str>>,
    >
    Whitespace<
        &'a str,
        (Output1, Output2, Output3, Output4, Output5, Output6),
        VerboseError<&'a str>,
    > for (A, B, C, D, E, F)
{
    fn parse(
        &mut self,
        input: &'a str,
    ) -> IResult<
        &'a str,
        (Output1, Output2, Output3, Output4, Output5, Output6),
        VerboseError<&'a str>,
    > {
        let (input, res1) = ws_parse(input, &mut self.0)?;
        let (input, res2) = ws_parse(input, &mut self.1)?;
        let (input, res3) = ws_parse(input, &mut self.2)?;
        let (input, res4) = ws_parse(input, &mut self.3)?;
        let (input, res5) = ws_parse(input, &mut self.4)?;
        let (input, res6) = ws_parse(input, &mut self.5)?;
        Ok((input, (res1, res2, res3, res4, res5, res6)))
    }
}

pub fn sep_many0<
    'a,
    Output1,
    Output2,
    A: Parser<&'a str, Output1, VerboseError<&'a str>>,
    B: Parser<&'a str, Output2, VerboseError<&'a str>>,
>(
    mut first_parser: A,
    mut many_parser: B,
) -> impl FnMut(&'a str) -> IResult<&'a str, (Output1, Vec<Output2>), VerboseError<&'a str>> {
    move |input: &'a str| {
        let (mut input, out1) = first_parser.parse(input)?;

        let mut many_out: Vec<Output2> = Vec::new();
        loop {
            let many_res = sep_parse(input, &mut many_parser);
            match many_res {
                Ok((next_input, out)) => {
                    many_out.push(out);
                    input = next_input;
                }
                Err(_) => break,
            }
        }

        Ok((input, (out1, many_out)))
    }
}

/// Parses a list of symbol-separated items.
///
/// Requires a separator between items, unless the parser is optional and does not match anything,
/// in which case the separator will not be consumed even if it is present.
///
/// Will also optionally consume a leading symbol separator.
pub fn sep<I: Clone, O, E: ParseError<I>, List: Sep<I, O, E>>(
    mut l: List,
) -> impl FnMut(I) -> IResult<I, O, E> {
    move |i: I| l.parse(i)
}

pub trait Sep<I, O, E> {
    /// Tests each parser with a symbol separator between them.
    fn parse(&mut self, input: I) -> IResult<I, O, E>;
}

fn sep_parse<'a, Output, A: Parser<&'a str, Output, VerboseError<&'a str>>>(
    input: &'a str,
    parser: &mut A,
) -> Result<(&'a str, Output), NomErr<VerboseError<&'a str>>> {
    let symbol_sep_res = symbol_separator(input);

    match symbol_sep_res {
        Ok((next_input, _)) => {
            match parser.parse(next_input) {
                Ok((parser_next_input, parser_res)) => {
                    let length = next_input.offset(parser_next_input);

                    // If the parser didn't capture anything, and there was a separator, uncapture the separator.
                    // Return the original input like nothing happened.
                    if length == 0 {
                        Ok((input, parser_res))
                    }
                    // If the parser did capture something, and there was a separator, we have a successful match.
                    else {
                        Ok((parser_next_input, parser_res))
                    }
                }
                Err(_) => Err(NomErr::Error(VerboseError::from_error_kind(
                    input,
                    nom::error::ErrorKind::Many0,
                ))),
            }
        }
        Err(e) => {
            // If there's no separator, this is only an error if the parser wanted to capture something.
            // i.e. If the parser is optional, it's not an error.
            match parser.parse(input) {
                Ok((parser_next_input, parser_res)) => {
                    let length = input.offset(parser_next_input);

                    // If the parser didn't capture anything, and there was no separator, this is not an error.
                    // Return the original input like nothing happened.
                    if length == 0 {
                        Ok((input, parser_res))
                    }
                    // If the parser captured something, but there was no separator, this is an error.
                    else {
                        Err(e)
                    }
                }
                Err(_) => {
                    // The separator is what really failed here, not the parser.
                    Err(e)
                }
            }
        }
    }
}

impl<
        'a,
        Output1,
        Output2,
        A: Parser<&'a str, Output1, VerboseError<&'a str>>,
        B: Parser<&'a str, Output2, VerboseError<&'a str>>,
    > Sep<&'a str, (Output1, Output2), VerboseError<&'a str>> for (A, B)
{
    fn parse(
        &mut self,
        input: &'a str,
    ) -> IResult<&'a str, (Output1, Output2), VerboseError<&'a str>> {
        let (input, _) = opt(symbol_separator)(input)?;
        let (input, res1) = self.0.parse(input)?;
        let (input, res2) = sep_parse(input, &mut self.1)?;
        Ok((input, (res1, res2)))
    }
}

impl<
        'a,
        Output1,
        Output2,
        Output3,
        A: Parser<&'a str, Output1, VerboseError<&'a str>>,
        B: Parser<&'a str, Output2, VerboseError<&'a str>>,
        C: Parser<&'a str, Output3, VerboseError<&'a str>>,
    > Sep<&'a str, (Output1, Output2, Output3), VerboseError<&'a str>> for (A, B, C)
{
    fn parse(
        &mut self,
        input: &'a str,
    ) -> IResult<&'a str, (Output1, Output2, Output3), VerboseError<&'a str>> {
        let (input, _) = opt(symbol_separator)(input)?;
        let (input, res1) = self.0.parse(input)?;
        let (input, res2) = sep_parse(input, &mut self.1)?;
        let (input, res3) = sep_parse(input, &mut self.2)?;
        Ok((input, (res1, res2, res3)))
    }
}

impl<
        'a,
        Output1,
        Output2,
        Output3,
        Output4,
        A: Parser<&'a str, Output1, VerboseError<&'a str>>,
        B: Parser<&'a str, Output2, VerboseError<&'a str>>,
        C: Parser<&'a str, Output3, VerboseError<&'a str>>,
        D: Parser<&'a str, Output4, VerboseError<&'a str>>,
    > Sep<&'a str, (Output1, Output2, Output3, Output4), VerboseError<&'a str>> for (A, B, C, D)
{
    fn parse(
        &mut self,
        input: &'a str,
    ) -> IResult<&'a str, (Output1, Output2, Output3, Output4), VerboseError<&'a str>> {
        let (input, _) = opt(symbol_separator)(input)?;
        let (input, res1) = self.0.parse(input)?;
        let (input, res2) = sep_parse(input, &mut self.1)?;
        let (input, res3) = sep_parse(input, &mut self.2)?;
        let (input, res4) = sep_parse(input, &mut self.3)?;
        Ok((input, (res1, res2, res3, res4)))
    }
}

impl<
        'a,
        Output1,
        Output2,
        Output3,
        Output4,
        Output5,
        A: Parser<&'a str, Output1, VerboseError<&'a str>>,
        B: Parser<&'a str, Output2, VerboseError<&'a str>>,
        C: Parser<&'a str, Output3, VerboseError<&'a str>>,
        D: Parser<&'a str, Output4, VerboseError<&'a str>>,
        E: Parser<&'a str, Output5, VerboseError<&'a str>>,
    > Sep<&'a str, (Output1, Output2, Output3, Output4, Output5), VerboseError<&'a str>>
    for (A, B, C, D, E)
{
    fn parse(
        &mut self,
        input: &'a str,
    ) -> IResult<&'a str, (Output1, Output2, Output3, Output4, Output5), VerboseError<&'a str>>
    {
        let (input, _) = opt(symbol_separator)(input)?;
        let (input, res1) = self.0.parse(input)?;
        let (input, res2) = sep_parse(input, &mut self.1)?;
        let (input, res3) = sep_parse(input, &mut self.2)?;
        let (input, res4) = sep_parse(input, &mut self.3)?;
        let (input, res5) = sep_parse(input, &mut self.4)?;
        Ok((input, (res1, res2, res3, res4, res5)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::{bytes::complete::tag, combinator::opt};

    #[test]
    fn ws_should_allow_whitespace_between() {
        // arrange
        let input = "hello world";

        // act
        let (next_input, res) = ws((tag("hello"), tag("world")))(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res, ("hello", "world"))
    }

    #[test]
    fn ws_should_allow_no_whitespace_between() {
        // arrange
        let input = "helloworld";

        // act
        let (next_input, res) = ws((tag("hello"), tag("world")))(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res, ("hello", "world"))
    }

    #[test]
    fn ws_should_should_not_match_trailing_whitespace() {
        // arrange
        let input = "hello world ";

        // act
        let (next_input, res) = ws((tag("hello"), tag("world")))(input).unwrap();

        // assert
        assert_eq!(next_input, " "); // trailing space left over
        assert_eq!(res, ("hello", "world"))
    }

    #[test]
    fn ws_should_allow_leading_whitespace() {
        // arrange
        let input = " hello world";

        // act
        let (next_input, res) = ws((tag("hello"), tag("world")))(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res, ("hello", "world"))
    }

    #[test]
    fn sep_should_allow_whitespace_between() {
        // arrange
        let input = "hello world";

        // act
        let (next_input, res) = sep((tag("hello"), tag("world")))(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res, ("hello", "world"))
    }

    #[test]
    fn sep_should_allow_multiple_whitespace_before_and_after() {
        // arrange
        let input = "hello  world";

        // act
        let (next_input, res) = sep((tag("hello"), tag("world")))(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res, ("hello", "world"))
    }

    #[test]
    fn sep_should_match_when_opt_doesnt() {
        // arrange
        let input = "hello bob";

        // act
        let (next_input, res) = sep((tag("hello"), opt(tag("world"))))(input).unwrap();

        // assert
        assert_eq!(next_input, " bob");
        assert_eq!(res, ("hello", None))
    }

    #[test]
    fn sep_many0_should_match_first() {
        // arrange
        let input = "hello";

        // act
        let (next_input, res) = sep_many0(tag("hello"), tag("world"))(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res, ("hello", vec![]))
    }

    #[test]
    fn sep_many0_should_match_multiple() {
        // arrange
        let input = "hello world world world";

        // act
        let (next_input, res) = sep_many0(tag("hello"), tag("world"))(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res, ("hello", vec!["world", "world", "world"]))
    }

    #[test]
    fn sep_many0_should_not_match_without_sep_symbol() {
        // arrange
        let input = "helloworld";

        // act
        let (next_input, res) = sep_many0(tag("hello"), tag("world"))(input).unwrap();

        // assert
        assert_eq!(next_input, "world");
        assert_eq!(res, ("hello", vec![]))
    }

    #[test]
    fn sep_many0_should_not_match_second_without_sep_symbol() {
        // arrange
        let input = "hello worldworldworld";

        // act
        let (next_input, res) = sep_many0(tag("hello"), tag("world"))(input).unwrap();

        // assert
        assert_eq!(next_input, "worldworld");
        assert_eq!(res, ("hello", vec!["world"]))
    }

    #[test]
    fn sep_many0_should_not_consume_trailing_sep_symbol() {
        // arrange
        let input = "hello world world world  ";

        // act
        let (next_input, res) = sep_many0(tag("hello"), tag("world"))(input).unwrap();

        // assert
        assert_eq!(next_input, "  ");
        assert_eq!(res, ("hello", vec!["world", "world", "world"]))
    }
}
