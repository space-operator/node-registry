use space_lib::{space, Result};

#[derive(Deserialize)]
struct Input {
    env: String,
}

#[space]
fn main(mut input: Input) -> Result<String> {
    Ok(std::env::var(input.env))
}
