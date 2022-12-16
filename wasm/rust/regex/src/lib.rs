use std::collections::HashMap;

use regex::{Error as RegexError, Regex};
use serde::Serialize;

use util::*;

#[cfg(not(test))]
#[no_mangle]
fn main(text: &String, regex: &String) -> Box<String> {
    iter_captures(text, regex)
}

#[derive(Serialize)]
struct Captures {
    r#match: String,
    unnamed: Vec<Option<String>>,
    named: HashMap<String, Option<String>>,
}

pub fn iter_captures(text: &String, regex: &String) -> Box<String> {
    serialize_result(iter_captures_impl(text, regex))
}

fn iter_captures_impl(text: &String, regex: &String) -> Result<Vec<Captures>, RegexError> {
    let re = Regex::new(regex)?;
    let names: Vec<_> = re.capture_names().flatten().collect();
    let len = re.captures_len();
    Ok(re
        .captures_iter(text)
        .map(|caps| Captures {
            r#match: caps.get(0).unwrap().as_str().to_owned(),
            unnamed: (1..len)
                .map(|j| caps.get(j).map(|item| item.as_str().to_owned()))
                .collect(),
            named: names
                .iter()
                .map(|&name| {
                    (
                        name.to_owned(),
                        caps.name(name).map(|item| item.as_str().to_owned()),
                    )
                })
                .collect(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::iter_captures;

    #[test]
    fn regex() {
        fn check(text: &str, regex: &str) -> String {
            *iter_captures(&text.to_owned(), &regex.to_owned())
        }

        assert_eq!(
            check("abcd", "a"),
            "{\
                \"status\":\"ok\",\
                \"value\":[{\"match\":\"a\",\"unnamed\":[],\"named\":{}}]\
            }"
        );
        assert_eq!(
            check("abcd", "e"),
            "{\
                \"status\":\"ok\",\
                \"value\":[]\
            }"
        );
        assert_eq!(
            check("abcd", "\\"),
            "{\
                \"status\":\"error\",\
                \"error\":\"regex parse error:\\n    \\\\\\n    ^\\n\
                    error: incomplete escape sequence, reached end of pattern prematurely\"\
            }"
        );
        assert_eq!(
            check(
                "@ a 123 @ b !bar 234 @ a foo 345",
                "@ ([a-z]+)(?: (?P<NamedGroup>[a-z]+))?(?: !([a-z]+))? ([0-9]+)"
            ),
            "{\
                \"status\":\"ok\",\
                \"value\":[\
                    {\
                        \"match\":\"@ a 123\",\
                        \"unnamed\":[\"a\",null,null,\"123\"],\
                        \"named\":{\"NamedGroup\":null}\
                    },{\
                        \"match\":\"@ b !bar 234\",\
                        \"unnamed\":[\"b\",null,\"bar\",\"234\"],\
                        \"named\":{\"NamedGroup\":null}\
                    },{\
                        \"match\":\"@ a foo 345\",\
                        \"unnamed\":[\"a\",\"foo\",null,\"345\"],\
                        \"named\":{\"NamedGroup\":\"foo\"}\
                    }\
                ]\
            }"
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
