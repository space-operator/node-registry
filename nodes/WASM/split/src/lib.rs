#[no_mangle]
extern fn main(input: &String, delimiter: &String) -> Box<Vec<String>> {
    Box::new(input.split(delimiter).map(str::trim).map(str::to_string).collect())
}
