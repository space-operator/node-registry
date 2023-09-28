use convert_case::{Case, Casing};
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
enum ConvertError {
    #[error("\"{0}\" is not a valid conversion target")]
    Type(String),
}

#[no_mangle]
fn main(input: &String, conversion_type: &String) -> Box<String> {
    let result = convert(input, conversion_type);
    util::serialize_result(result)
}

fn convert(input: &String, output_type: &String) -> Result<String, ConvertError> {
    let case = match output_type.as_str() {
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

        _ => return Err(ConvertError::Type(output_type.clone())),
    };

    Ok(input.to_case(case))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_to_snake_case() {
        let input = "This is a test for all conversion types".to_string();

        let result = convert(&input, &"snake".to_string());
        assert_eq!(
            result.unwrap(),
            "this_is_a_test_for_all_conversion_types".to_string()
        );

        let result = convert(&input, &"upper_snake".to_string());
        assert_eq!(
            result.unwrap(),
            "THIS_IS_A_TEST_FOR_ALL_CONVERSION_TYPES".to_string()
        );

        let result = convert(&input, &"screaming_snake".to_string());
        assert_eq!(
            result.unwrap(),
            "THIS_IS_A_TEST_FOR_ALL_CONVERSION_TYPES".to_string()
        );

        let result = convert(&input, &"kebab".to_string());
        assert_eq!(
            result.unwrap(),
            "this-is-a-test-for-all-conversion-types".to_string()
        );

        let result = convert(&input, &"upper_kebab".to_string());
        assert_eq!(
            result.unwrap(),
            "THIS-IS-A-TEST-FOR-ALL-CONVERSION-TYPES".to_string()
        );

        let result = convert(&input, &"train".to_string());
        assert_eq!(
            result.unwrap(),
            "This-Is-A-Test-For-All-Conversion-Types".to_string()
        );

        let result = convert(&input, &"flat".to_string());
        assert_eq!(
            result.unwrap(),
            "thisisatestforallconversiontypes".to_string()
        );

        let result = convert(&input, &"upper_flat".to_string());
        assert_eq!(
            result.unwrap(),
            "THISISATESTFORALLCONVERSIONTYPES".to_string()
        );

        let result = convert(&input, &"alternating".to_string());
        assert_eq!(
            result.unwrap(),
            "tHiS iS a TeSt FoR aLl CoNvErSiOn TyPeS".to_string()
        );

        let result = convert(&input, &"upper".to_string());
        assert_eq!(
            result.unwrap(),
            "THIS IS A TEST FOR ALL CONVERSION TYPES".to_string()
        );

        let result = convert(&input, &"lower".to_string());
        assert_eq!(
            result.unwrap(),
            "this is a test for all conversion types".to_string()
        );

        let result = convert(&input, &"title".to_string());
        assert_eq!(
            result.unwrap(),
            "This Is A Test For All Conversion Types".to_string()
        );

        let result = convert(&input, &"toggle".to_string());
        assert_eq!(
            result.unwrap(),
            "tHIS iS a tEST fOR aLL cONVERSION tYPES".to_string()
        );

        let result = convert(&input, &"camel".to_string());
        assert_eq!(
            result.unwrap(),
            "thisIsATestForAllConversionTypes".to_string()
        );

        let result = convert(&input, &"pascal".to_string());
        assert_eq!(
            result.unwrap(),
            "ThisIsATestForAllConversionTypes".to_string()
        );

        let result = convert(&input, &"upper_camel".to_string());
        assert_eq!(
            result.unwrap(),
            "ThisIsATestForAllConversionTypes".to_string()
        );

        let result = convert(&input, &"cobol".to_string());
        assert_eq!(
            result.unwrap(),
            "THIS-IS-A-TEST-FOR-ALL-CONVERSION-TYPES".to_string()
        );

        let result = convert(&input, &"invalid".to_string());
        assert!(result.is_err());
    }
}

mod util {
    use std::fmt::Display;

    use serde::{Serialize, Serializer};

    #[derive(Serialize)]
    #[serde(tag = "status", rename_all = "lowercase")]
    #[serde(bound(serialize = "T: Serialize, E: Display"))]
    pub enum ResultSer<T, E> {
        Ok {
            value: T,
        },
        Error {
            #[serde(serialize_with = "use_display")]
            error: E,
        },
    }

    pub fn serialize_result<T, E>(result: Result<T, E>) -> Box<String>
    where
        T: Serialize,
        E: Display,
    {
        let result = match result {
            Ok(value) => ResultSer::Ok { value },
            Err(error) => ResultSer::Error { error },
        };
        Box::new(serde_json::to_string(&result).unwrap())
    }

    pub fn use_display<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }
}
