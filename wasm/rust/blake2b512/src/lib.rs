use space_lib::space;
use blake2::{Blake2b512, Digest};

#[space]
fn main(input: String) -> String {
    let mut hasher = Blake2b512::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();
    hex::encode(hash)
}
