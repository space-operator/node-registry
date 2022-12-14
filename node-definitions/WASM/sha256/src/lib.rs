use sha2::{Sha256, Digest};

#[no_mangle]
extern fn main(input: &String) -> Box<String> {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();
    Box::new(hex::encode(hash))
}
