use sha2::{Sha256, Digest};
use space_lib::space;

#[space]
fn main(input: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();
    hex::encode(hash)
}
