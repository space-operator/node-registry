use space_lib::space;

#[space]
fn main(mut input: Vec<i64>) -> Vec<i64> {
    input.sort();
    input.reverse();
    input
}
