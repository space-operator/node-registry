use space_lib::space;
use serde::Deserialize;

#[derive(Deserialize)]
struct Input {
    left: String,
    right: String,
}

#[space]
fn main(Input { left, right }: Input) -> Vec<String> {
    if left.lines().count() == 1 && right.lines().count() == 1 {
        diff::chars(&left, &right).into_iter().filter_map(|result| match result {
            diff::Result::Left(left) => Some(format!("-{left}")),
            diff::Result::Right(right) => Some(format!("+{right}")),
            _ => None,
        }).collect()
    } else {
        diff::lines(&left, &right).into_iter().filter_map(|result| match result {
            diff::Result::Left(left) => Some(format!("-{left}")),
            diff::Result::Right(right) => Some(format!("+{right}")),
            _ => None,
        }).collect()
    }
}
