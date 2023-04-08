#[cfg(not(test))]
#[no_mangle]
fn main(text: &String, regex: &String) -> Box<String> {
    sort(text, regex)
}

use std::cmp::Ordering;

use serde_json::{Number, Value};
use thiserror::Error as ThisError;

use util::*;

enum SortOrdering {
    Ascending,
    Descending,
}

#[derive(Debug, ThisError)]
enum SortError {
    #[error("first argument can not be deserialized as array")]
    Decode(serde_json::Error),
    #[error("failed to parse \"{0}\" as ordering, valid values are \"asc\" and \"desc\"")]
    Ordering(String),
    #[error(transparent)]
    Compare(#[from] CompareError),
}

pub fn sort(array: &String, ordering: &String) -> Box<String> {
    serialize_result(sort_impl(array, ordering))
}

fn sort_impl(array: &String, ordering: &String) -> Result<Vec<Value>, SortError> {
    let mut array: Vec<Value> = serde_json::from_str(array).map_err(SortError::Decode)?;
    let ordering = match ordering.as_str() {
        "asc" => SortOrdering::Ascending,
        "desc" => SortOrdering::Descending,
        ordering => Err(SortError::Ordering(ordering.to_owned()))?,
    };
    // fallible sorting
    let mut err = None;
    array.sort_by(|lhs, rhs| match &err {
        Some(_err) => Ordering::Equal,
        None => match compare(lhs, rhs) {
            Ok(ord) => match ordering {
                SortOrdering::Ascending => ord,
                SortOrdering::Descending => ord.reverse(),
            },
            Err(new_err) => {
                err = Some(new_err);
                Ordering::Equal
            }
        },
    });
    match err {
        Some(err) => Err(err)?,
        None => Ok(array),
    }
}

#[derive(Debug, ThisError)]
enum CompareError {
    #[error("failed to compare numbers: \"{0}\" and \"{1}\"")]
    Number(Number, Number),
    #[error("array with arrays can not be sorted")]
    Array,
    #[error("array with objects can not be sorted")]
    Object,
    #[error("mixed array can not be sorted")]
    Mixed,
}

fn compare(lhs: &Value, rhs: &Value) -> Result<Ordering, CompareError> {
    match (lhs, rhs) {
        (Value::Null, Value::Null) => Ok(Ordering::Equal),
        (Value::Bool(lhs), Value::Bool(rhs)) => Ok(lhs.cmp(rhs)),
        (Value::Number(lhs), Value::Number(rhs)) => {
            if let (Some(lhs), Some(rhs)) = (lhs.as_u64(), rhs.as_u64()) {
                Ok(lhs.cmp(&rhs))
            } else if let (Some(lhs), Some(rhs)) = (lhs.as_i64(), rhs.as_i64()) {
                Ok(lhs.cmp(&rhs))
            } else if let (Some(flhs), Some(frhs)) = (lhs.as_f64(), rhs.as_f64()) {
                flhs.partial_cmp(&frhs)
                    .ok_or(CompareError::Number(lhs.clone(), rhs.clone()))
            } else {
                Err(CompareError::Number(lhs.clone(), rhs.clone()))
            }
        }
        (Value::String(lhs), Value::String(rhs)) => Ok(lhs.cmp(rhs)),
        (Value::Array(_), _) | (_, Value::Array(_)) => Err(CompareError::Array),
        (Value::Object(_), _) | (_, Value::Object(_)) => Err(CompareError::Object),
        (Value::Null, _)
        | (_, Value::Null)
        | (Value::Bool(_), _)
        | (_, Value::Bool(_))
        | (Value::Number(_), _)
        | (_, Value::Number(_)) => Err(CompareError::Mixed),
    }
}

#[cfg(test)]
mod tests {
    use super::sort;

    #[test]
    fn regex() {
        fn check(array: &str, ordering: &str) -> String {
            *sort(&array.to_owned(), &ordering.to_owned())
        }

        assert_eq!(
            check("abcd", "foo"),
            "{\"status\":\"error\",\"error\":\"first argument can not be deserialized as array\"}"
        );
        assert_eq!(
            check("[1, null, {}]", "foo"),
            "{\
                \"status\":\"error\",\
                \"error\":\"failed to parse \\\"foo\\\" as ordering, \
                    valid values are \\\"asc\\\" and \\\"desc\\\"\"\
            }"
        );
        assert_eq!(
            check("[1, null, {}]", "asc"),
            "{\"status\":\"error\",\"error\":\"array with objects can not be sorted\"}"
        );
        assert_eq!(
            check("[1, null, []]", "asc"),
            "{\"status\":\"error\",\"error\":\"array with arrays can not be sorted\"}"
        );
        assert_eq!(
            check("[1, null, \"abc\"]", "asc"),
            "{\"status\":\"error\",\"error\":\"mixed array can not be sorted\"}"
        );
        assert_eq!(
            check("[1, 2, 3]", "asc"),
            "{\"status\":\"ok\",\"value\":[1,2,3]}"
        );
        assert_eq!(
            check("[10, 20, 8, 22, 1, 29, 5, 25]", "asc"),
            "{\"status\":\"ok\",\"value\":[1,5,8,10,20,22,25,29]}"
        );
        assert_eq!(
            check("[10, 20, 8, 22, 1, 29, 5, 25]", "desc"),
            "{\"status\":\"ok\",\"value\":[29,25,22,20,10,8,5,1]}"
        );
        assert_eq!(
            check("[null, null, null]", "asc"),
            "{\"status\":\"ok\",\"value\":[null,null,null]}"
        );
        assert_eq!(
            check("[null, null, null]", "desc"),
            "{\"status\":\"ok\",\"value\":[null,null,null]}"
        );
        assert_eq!(
            check("[10.1, 20, 8, 22.1, 1, 29, 5, 25]", "asc"),
            "{\"status\":\"ok\",\"value\":[1,5,8,10.1,20,22.1,25,29]}"
        );
        assert_eq!(
            check("[10.1, 20, 8, 22.1, 1, 29, 5, 25]", "desc"),
            "{\"status\":\"ok\",\"value\":[29,25,22.1,20,10.1,8,5,1]}"
        );
        assert_eq!(
            check("[\"foo\", \"2\", \"wow\", \"10\", \"bar\", \"3\"]", "asc"),
            "{\"status\":\"ok\",\"value\":[\"10\",\"2\",\"3\",\"bar\",\"foo\",\"wow\"]}"
        );
        assert_eq!(
            check("[\"foo\", \"2\", \"wow\", \"10\", \"bar\", \"3\"]", "desc"),
            "{\"status\":\"ok\",\"value\":[\"wow\",\"foo\",\"bar\",\"3\",\"2\",\"10\"]}"
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
