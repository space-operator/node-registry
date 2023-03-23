use serde::Deserialize;
use space_lib::{space, Result};
use convert_case::{Case, Casing};

#[derive(Deserialize)]
struct Input {
    value: String,
    case_type: String,
}

#[space]
fn main(input: Input) -> Result<String> {
    let case = match input.case_type.as_str() {
        "upper" => Case::Upper,
        "lower" => Case::Lower,
        "title" => Case::Title,
        "toggle" => Case::Toggle,
        "camel" => Case::Camel,
        "pascal" => Case::Pascal,
        "upper_camel" => Case::UpperCamel,
        "snake" => Case::Snake,
        "upper_snake" => Case::UpperSnake,
        "screaming_snake" => Case::ScreamingSnake,
        "kebab" => Case::Kebab,
        "cobol" => Case::Cobol,
        "upper_kebab" => Case::UpperKebab,
        "train" => Case::Train,
        "flat" => Case::Flat,
        "upper_flat" => Case::UpperFlat,
        "alternating" => Case::Alternating,
        _ => Err(format!("Invalid case: {}", input.case_type))?,
    };
    Ok(input.value.to_case(case))
}
