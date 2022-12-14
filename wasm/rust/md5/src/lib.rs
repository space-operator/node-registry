#[no_mangle]
extern fn main(input: &String) -> Box<String> {
    let hash = md5::compute(input.as_bytes());
    Box::new(hex::encode(*hash))
}
