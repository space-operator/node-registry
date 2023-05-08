use rand::Rng;
use space_lib::space;
use serde::Deserialize;

#[derive(Deserialize)]
struct Input {
    min: u64,
    max: u64,
    len: u64,
}

#[space]
fn main(input: Input) -> Vec<u64> {
    let mut rng = rand::thread_rng();
    (0..input.len).map(|_| rng.gen_range(input.min..input.max)).collect()
}
