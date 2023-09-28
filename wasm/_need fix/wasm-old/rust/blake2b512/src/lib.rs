use blake2::{Blake2b512, Digest};

#[no_mangle]
extern fn main(input: &String) -> Box<String> {
    let mut hasher = Blake2b512::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();
    Box::new(hex::encode(hash))
}
