use convert_case::{Case, Casing};

#[no_mangle]
fn main(input: &String) -> Box<String> {
    let result = input.to_case(Case::Upper);
    Box::new(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_to_snake_case() {
        let input = "uppercase".to_string();
        let result = *main(&input);
        assert_eq!(*result, "UPPERCASE".to_string());
    }
}
