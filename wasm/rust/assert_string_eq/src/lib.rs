use serde::Deserialize;
use space_lib::{space, Result};

#[derive(Deserialize)]
struct Input {
    input: String,
    expected: String,
}

#[space]
fn main(input: Input) -> Result<String> {
    if input.input == input.expected {
        Ok(input.input)
    } else {
        Err("Input and expected do not match")?
    }
}
