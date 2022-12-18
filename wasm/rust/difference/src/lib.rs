use serde_json::json;
use util::serialize_result;

#[no_mangle]
fn main(left: &String, right: &String) -> Box<String> {
    serialize_result(diff(left, right))
}

fn diff(left: &str, right: &str) -> Result<Box<String>, String> {
    // check if the text is a single line or multiple lines
    let lines1 = left.lines().count();
    let lines2 = right.lines().count();

    let mut result = Vec::new();

    if lines1 == 1 && lines2 == 1 {
        // single line diff so we can use chars
        for diff in diff::chars(left, right) {
            match diff {
                diff::Result::Left(l) => {
                    result.push(format!("-{}", l));
                }
                diff::Result::Both(_, _) => (),
                diff::Result::Right(r) => {
                    result.push(format!("+{}", r));
                }
            }
        }
    } else {
        // multi line diff so we can use lines
        for diff in diff::lines(left, right) {
            match diff {
                diff::Result::Left(l) => {
                    result.push(format!("-{}", l));
                }
                diff::Result::Both(_, _) => (),
                diff::Result::Right(r) => {
                    result.push(format!("+{}", r));
                }
            }
        }
    }

    Ok(Box::new(json!(result).to_string()))
}

#[cfg(test)]
mod tests {
    use crate::diff;

    #[test]
    fn test_diff() {
        let left = "hello world";
        let right = "hello world";
        let result = *diff(left, right).unwrap();
        assert_eq!(result, "[]");

        let left = "hello world";
        let right = "hello world!";
        let result = *diff(left, right).unwrap();
        assert_eq!(result, r#"["+!"]"#);

        let left = "hello world";
        let right = "hllo wold!";
        let result = *diff(left, right).unwrap();
        "[\"-e\",\"-r\",\"+!\"]";
        assert_eq!(result, r#"["-e","-r","+!"]"#);

        // test multi line

        let left = "hello \n world";
        let right = "hello \n world";
        let result = *diff(left, right).unwrap();
        assert_eq!(result, "[]");

        let left = "hello \n world";
        let right = "hello \n wrld!";

        let result = *diff(left, right).unwrap();
        assert_eq!(result, r#"["- world","+ wrld!"]"#);

        let left = "hello \n world";
        let right = "hllo \n wold!";

        let result = *diff(left, right).unwrap();
        assert_eq!(result, r#"["-hello ","- world","+hllo ","+ wold!"]"#);
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
