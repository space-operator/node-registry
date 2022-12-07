#[no_mangle]
fn main(input: &String) -> Box<String> {
    Box::new(input.chars().rev().collect())
}
