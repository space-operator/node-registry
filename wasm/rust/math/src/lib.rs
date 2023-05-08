use serde::Deserialize;
use space_lib::{space, Result};
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Input {
    expr: String,
    vars: BTreeMap<String, f64>,
}

#[space]
fn main(mut input: Input) -> Result<f64> {
    Ok(fasteval::ez_eval(&input.expr, &mut input.vars)?)
}
