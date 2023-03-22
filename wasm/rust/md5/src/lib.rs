use space_lib::space;

#[space]
fn main(input: String) -> String {
    let hash = md5::compute(input.as_bytes());
    hex::encode(*hash)
}
