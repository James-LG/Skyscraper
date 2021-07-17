mod tokenizer;

use std::error::Error;

pub fn parse(text: &str) -> Result<(), Box<dyn Error>> {
    tokenizer::lex(text)?;
    Ok(())
}