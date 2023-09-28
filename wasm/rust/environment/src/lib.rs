use space_lib::space;

#[space]
fn main(input: String) -> Option<String> {
    dbg!(std::env::vars());
    
    std::env::var(input).ok()
}
