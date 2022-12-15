use convert_case::{Case, Casing};

#[no_mangle]
fn main(input: &String) -> Box<String> {
    let result = input.to_case(Case::Camel);
    Box::new(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_to_camel_case() {
        let input = "camel_case".to_string();
        let result = *main(&input);
        assert_eq!(*result, "camelCase".to_string());
    }
}
