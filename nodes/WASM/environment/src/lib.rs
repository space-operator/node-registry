#[no_mangle]
extern "C" fn main(input: &String) -> Option<Box<String>> {
    std::env::var(input).ok().map(Box::new)
}
