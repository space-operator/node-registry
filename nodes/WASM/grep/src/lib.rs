#[no_mangle]
extern fn main(input: &String, pattern: &String) -> Box<String> {
    Box::new(input.lines().filter(|it| it.contains(pattern)).collect())
}
