use getrandom::getrandom;
use serde::Deserialize;
use space_lib::{space, Result};

#[derive(Deserialize)]
struct Input {
    start: u64,
    end: u64,
}

#[space]
fn main(input: Input) -> Result<u64> {
    let mut buffer = [0; 8];
    getrandom(&mut buffer)?;
    Ok(u64::from_le_bytes(buffer) % input.end.saturating_sub(input.start) + input.start)
}
