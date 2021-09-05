use std::error::Error;

use crate::{Xpath, XpathElement, XpathQuery, tokenizer::{self, Symbol}};

pub fn parse(text: &str) -> Result<Xpath, Box<dyn Error>> {
    let symbols = tokenizer::lex(text)?;
    let mut elements: Vec<XpathElement> = Vec::new();

    let mut open_square_bracket = false;
    for symbol in symbols {
        match symbol {
            Symbol::Slash => elements.push(XpathElement::SearchRoot),
            Symbol::DoubleSlash => elements.push(XpathElement::SearchAll),
            Symbol::OpenSquareBracket => {
                if let Some(el) = elements.last() {
                    if let XpathElement::Element(_) = el {
                        open_square_bracket = true;
                    } else {
                        return Err("Open square bracket must immediately follow an element".into());
                    }
                } else {
                    return Err("Open square bracket cannot be first element".into());
                }
            },
            Symbol::CloseSquareBracket => {
                if open_square_bracket {
                    open_square_bracket = false;
                } else {
                    return Err("Close square bracket has no matching opening square bracket".into());
                }
            },
            Symbol::OpenBracket => todo!(),
            Symbol::CloseBracket => todo!(),
            Symbol::Wildcard => todo!(),
            Symbol::Dot => todo!(),
            Symbol::DoubleDot => todo!(),
            Symbol::AssignmentSign => todo!(),
            Symbol::AtSign => todo!(),
            Symbol::MinusSign => todo!(),
            Symbol::AddSign => todo!(),
            Symbol::GreaterThanSign => todo!(),
            Symbol::LessThanSign => todo!(),
            Symbol::Identifier(identifier) => elements.push(XpathElement::Element(XpathQuery::new(identifier))),
            Symbol::Text(_) => todo!(),
            Symbol::Number(_) => todo!(),
        }
    }

    Ok(Xpath { elements })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_works() {
        let text = "//book/title";

        let result = parse(text).unwrap();

        let expected = vec![
            XpathElement::SearchAll,
            XpathElement::Element(XpathQuery::new(String::from("book"))),
            XpathElement::SearchRoot,
            XpathElement::Element(XpathQuery::new(String::from("title")))
        ];

        // looping makes debugging much easier than just asserting the entire vectors are equal
        for (e, r) in expected.into_iter().zip(result.elements) {
            assert_eq!(e, r);
        }
    }
}
