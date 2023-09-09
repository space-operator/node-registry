use space_lib::space;

#[space]
fn main(input: String) -> Option<String> {
    std::env::var(input).ok()
}
