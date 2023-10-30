use std::ops::RangeFrom;

use nom::{
    error::{ErrorKind, ParseError, VerboseError},
    AsChar, Err as NomErr, IResult, InputIter, Offset, Parser, Slice,
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

pub fn max<I: Clone, O, E: ParseError<I>, List: Max<I, O, E>>(
    mut l: List,
) -> impl FnMut(I) -> IResult<I, O, E> {
    move |i: I| l.choice(i)
}

pub trait Max<I, O, E> {
    /// Tests each parser in the tuple and returns the result of the first one that succeeds
    fn choice(&mut self, input: I) -> IResult<I, O, E>;
}

impl<
        Input: Clone + Offset,
        Output,
        Error: ParseError<Input>,
        A: Parser<Input, Output, Error>,
        B: Parser<Input, Output, Error>,
    > Max<Input, Output, Error> for (A, B)
{
    fn choice(&mut self, input: Input) -> IResult<Input, Output, Error> {
        let res0 = self.0.parse(input.clone());
        let res1 = self.1.parse(input.clone());

        fun_name(res0, res1, input)
    }
}

fn fun_name<Input: Clone + Offset, Output, Error: ParseError<Input>>(
    res0: Result<(Input, Output), NomErr<Error>>,
    res1: Result<(Input, Output), NomErr<Error>>,
    input: Input,
) -> Result<(Input, Output), NomErr<Error>> {
    match (res0, res1) {
        (Ok(res0), Ok(res1)) => {
            let length0 = input.offset(&res0.0);
            let length1 = input.offset(&res1.0);

            if length0 >= length1 {
                Ok(res0)
            } else {
                Ok(res1)
            }
        }
        (Ok(res0), Err(_)) => Ok(res0),
        (Err(_), Ok(res1)) => Ok(res1),
        (Err(err0), Err(err1)) => match (err0, err1) {
            (NomErr::Error(err0), NomErr::Error(err1)) => {
                let combined_err = err0.or(err1);
                Err(NomErr::Error(Error::append(
                    input,
                    ErrorKind::Alt,
                    combined_err,
                )))
            }
            (err0, _err1) => Err(err0),
        },
    }
}

impl<
        Input: Clone + Offset,
        Output,
        Error: ParseError<Input>,
        A: Parser<Input, Output, Error>,
        B: Parser<Input, Output, Error>,
        C: Parser<Input, Output, Error>,
        D: Parser<Input, Output, Error>,
        E: Parser<Input, Output, Error>,
    > Max<Input, Output, Error> for (A, B, C, D, E)
{
    fn choice(&mut self, input: Input) -> IResult<Input, Output, Error> {
        let res0 = self.0.parse(input.clone());

        let res1 = self.1.parse(input.clone());
        let total_res = fun_name(res0, res1, input.clone());

        let res2 = self.2.parse(input.clone());
        let total_res = fun_name(total_res, res2, input.clone());

        let res3 = self.3.parse(input.clone());
        let total_res = fun_name(total_res, res3, input.clone());

        let res4 = self.4.parse(input.clone());
        let total_res = fun_name(total_res, res4, input.clone());

        total_res
    }
}

impl<
        Input: Clone + Offset,
        Output,
        Error: ParseError<Input>,
        A: Parser<Input, Output, Error>,
        B: Parser<Input, Output, Error>,
        C: Parser<Input, Output, Error>,
        D: Parser<Input, Output, Error>,
        E: Parser<Input, Output, Error>,
        F: Parser<Input, Output, Error>,
        G: Parser<Input, Output, Error>,
        H: Parser<Input, Output, Error>,
        I: Parser<Input, Output, Error>,
    > Max<Input, Output, Error> for (A, B, C, D, E, F, G, H, I)
{
    fn choice(&mut self, input: Input) -> IResult<Input, Output, Error> {
        let res0 = self.0.parse(input.clone());

        let res1 = self.1.parse(input.clone());
        let total_res = fun_name(res0, res1, input.clone());

        let res2 = self.2.parse(input.clone());
        let total_res = fun_name(total_res, res2, input.clone());

        let res3 = self.3.parse(input.clone());
        let total_res = fun_name(total_res, res3, input.clone());

        let res4 = self.4.parse(input.clone());
        let total_res = fun_name(total_res, res4, input.clone());

        let res5 = self.5.parse(input.clone());
        let total_res = fun_name(total_res, res5, input.clone());

        let res6 = self.6.parse(input.clone());
        let total_res = fun_name(total_res, res6, input.clone());

        let res7 = self.7.parse(input.clone());
        let total_res = fun_name(total_res, res7, input.clone());

        let res8 = self.8.parse(input.clone());
        let total_res = fun_name(total_res, res8, input.clone());

        total_res
    }
}

#[cfg(test)]
mod tests {
    use nom::bytes::complete::tag;

    use super::*;

    #[test]
    fn max_should_choose_option_that_matches_most() {
        // arrange
        let input = "hello world";

        // act
        let result: Result<(&str, &str), NomErr<VerboseError<&str>>> =
            max((tag("hello"), tag("hello world")))(input);
        let (next_input, res) = result.unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res, input);
    }
}
