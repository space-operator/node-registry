#[cfg(not(test))]
#[no_mangle]
fn main(text: &String, regex: &String) -> Box<String> {
    math(text, regex)
}

use std::collections::BTreeMap;

use thiserror::Error as ThisError;

use util::*;

#[derive(Debug, ThisError)]
enum MathError {
    #[error("second argument can not be deserialized an object of values")]
    Decode(serde_json::Error),
    #[error(transparent)]
    Compute(#[from] fasteval::Error),
}

pub fn math(expr: &String, vars: &String) -> Box<String> {
    serialize_result(math_impl(expr, vars))
}

fn math_impl(expr: &String, vars: &String) -> Result<f64, MathError> {
    let mut vars: BTreeMap<String, f64> = serde_json::from_str(vars).map_err(MathError::Decode)?;
    Ok(fasteval::ez_eval(expr, &mut vars)?)
}

#[cfg(test)]
mod tests {
    use super::math;

    #[test]
    fn regex() {
        fn check(array: &str, ordering: &str) -> String {
            *math(&array.to_owned(), &ordering.to_owned())
        }

        assert_eq!(
            check("qwe", "qwe"),
            "{\"status\":\"error\",\"error\":\"second argument can not be deserialized an object of values\"}"
        );
        assert_eq!(
            check("qwe", "{}"),
            "{\"status\":\"error\",\"error\":\"Undefined(\\\"qwe\\\")\"}"
        );
        assert_eq!(
            check("1 + 2 + 3", "{}"),
            "{\"status\":\"ok\",\"value\":6.0}"
        );
        assert_eq!(
            check("round(0.0001, pi()) + 1000", "{}"),
            "{\"status\":\"ok\",\"value\":1003.1416}"
        );
        assert_eq!(
            check("x + y", "{\"x\": 100, \"y\": 20}"),
            "{\"status\":\"ok\",\"value\":120.0}"
        );
        assert_eq!(
            check("(sin(x)^2 + cos(x)^2)^0.5", "{\"x\": 123456789}"),
            "{\"status\":\"ok\",\"value\":1.0}"
        );
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
