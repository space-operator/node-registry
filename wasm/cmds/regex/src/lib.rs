use regex::Regex;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
enum IsMatchResult {
    Ok { value: bool },
    Error { error: String },
}

#[no_mangle]
fn is_match(text: &String, regex: &String) -> Box<String> {
    fn merge(a: &mut Value, b: Value) {
        match (a, b) {
            (a @ &mut Value::Object(_), Value::Object(b)) => {
                let a = a.as_object_mut().unwrap();
                for (k, v) in b {
                    merge(a.entry(k).or_insert(Value::Null), v);
                }
            }
            (a, b) => *a = b,
        }
    }

    let re = Regex::new(regex);

    let is_match_result = match &re {
        Ok(re) => IsMatchResult::Ok {
            value: re.is_match(text),
        },
        Err(err) => IsMatchResult::Error {
            error: err.to_string(),
        },
    };
    let is_match = serde_json::to_string(&is_match_result).unwrap();

    let capture_group = re
        .unwrap()
        .find_iter(text)
        .map(|m| m.as_str())
        .collect::<Vec<_>>();

    let mut captures = serde_json::to_value(&capture_group).unwrap();
    let res = merge(
        &mut captures,
        serde_json::to_value(&is_match_result).unwrap(),
    );
    dbg!(res);
    Box::new(serde_json::to_string(&is_match_result).unwrap())
}

#[cfg(test)]
mod tests {
    fn is_match(text: &str, regex: &str) -> String {
        *super::is_match(&text.to_owned(), &regex.to_owned())
    }

    #[test]
    fn regex() {
        assert_eq!(is_match("this is some test", r"[a-z]+"), "123");
        // assert_eq!(is_match("abcd", "e"), "{\"status\":\"ok\",\"value\":false}");
        // assert_eq!(is_match("abcd", "\\"), "{\"status\":\"error\",\"error\":\"regex parse error:\\n    \\\\\\n    ^\\nerror: incomplete escape sequence, reached end of pattern prematurely\"}");
    }
}
