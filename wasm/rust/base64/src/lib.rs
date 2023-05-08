use space_lib::space;

#[space]
fn main(input: String) -> String {
    base64::encode(input.as_bytes())
}
