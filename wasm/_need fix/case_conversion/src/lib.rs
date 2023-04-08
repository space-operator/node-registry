use space_lib::space;
use serde::Deserialize;
use convert_case::Casing;

#[derive(Deserialize)]
pub enum Case {
    Upper,
    Lower,
    Title,
    Toggle,
    Camel,
    Pascal,
    UpperCamel,
    Snake,
    UpperSnake,
    ScreamingSnake,
    Kebab,
    Cobol,
    UpperKebab,
    Train,
    Flat,
    UpperFlat,
    Alternating,
    Random,
    PseudoRandom,
}

#[derive(Deserialize)]
struct Input {
    value: String,
    case_type: Case,
}

#[space]
fn main(input: Input) -> String {
    let case = unsafe { std::mem::transmute(input.case_type) };
    input.value.to_case(case)
}
