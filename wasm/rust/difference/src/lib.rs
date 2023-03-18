use space_lib::space;
use serde::Deserialize;

#[derive(Deserialize)]
struct Input {
    left: String,
    right: String,
}

#[space]
fn main(input: Input) -> Vec<String> {
    if input.left.lines().count() == 1 && input.right.lines().count() == 1 {
        diff::chars(&input.left, &input.right).into_iter().filter_map(|result| match result {
            diff::Result::Left(left) => Some(format!("-{left}")),
            diff::Result::Right(right) => Some(format!("+{right}")),
            _ => None,
        }).collect()
    } else {
        diff::lines(&input.left, &input.right).into_iter().filter_map(|result| match result {
            diff::Result::Left(left) => Some(format!("-{left}")),
            diff::Result::Right(right) => Some(format!("+{right}")),
            _ => None,
        }).collect()
    }
}
