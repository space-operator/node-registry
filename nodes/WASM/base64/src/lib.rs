#[no_mangle]
extern "C" fn main(input: &String) -> Box<String> {
    let hash = base64::encode(input.as_bytes());
    Box::new(hash)
}
