use serde::{Deserialize, Serialize};
use url::{Host, Position, Url};

#[derive(Serialize, Deserialize, Debug)]
pub struct UrlObject {
    scheme: String,
    username: String,
    password: Option<String>,
    host_str: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    path: String,
    path_segments: Option<Vec<String>>,
    query: Option<String>,
    before_path: String,
    fragment: Option<String>,
    cannot_be_a_base: bool,
}

#[no_mangle]
fn main(url: &String) -> Box<String> {
    let parsed_url = Url::parse(url).unwrap();
    let to_string = |s: &str| s.to_string();
    let url_object = UrlObject {
        scheme: parsed_url.scheme().to_string(),
        username: parsed_url.username().to_string(),
        password: parsed_url.password().map(to_string),
        host_str: parsed_url.host_str().map(to_string),
        host: if let Some(Host::Domain(h)) = parsed_url.host() {
            Some(h.to_string())
        } else {
            None
        },
        port: parsed_url.port(),
        path: parsed_url.path().to_string(),
        path_segments: if parsed_url.cannot_be_a_base() {
            None
        } else {
            Some(parsed_url.path_segments().unwrap().map(to_string).collect())
        },
        query: parsed_url.query().map(to_string),
        before_path: parsed_url[Position::BeforePath..].to_string(),
        fragment: parsed_url.fragment().map(to_string),
        cannot_be_a_base: parsed_url.cannot_be_a_base(),
    };

    let url_serialized = serde_json::to_string(&url_object).unwrap();
    Box::new(url_serialized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_field_parsing_works() {
        let my_url =
            main(&"https://github.com/rust-lang/rust/issues?labels=E-easy&state=open".to_string());
        let my_url: UrlObject = serde_json::from_str(&my_url).unwrap();
        assert_eq!(my_url.scheme, "https".to_string());
        assert_eq!(my_url.username, "");
        assert_eq!(my_url.password, None);
        assert_eq!(my_url.host_str, Some("github.com".to_string()));
        assert_eq!(my_url.port, None);
        assert_eq!(my_url.path, "/rust-lang/rust/issues".to_string());
        assert_eq!(
            my_url.path_segments,
            Some(vec![
                "rust-lang".to_string(),
                "rust".to_string(),
                "issues".to_string()
            ])
        );
        assert_eq!(my_url.query, Some("labels=E-easy&state=open".to_string()));
        assert_eq!(
            my_url.before_path,
            "/rust-lang/rust/issues?labels=E-easy&state=open".to_string()
        );
        assert_eq!(my_url.fragment, None);
        assert_eq!(my_url.cannot_be_a_base, false);
    }
}
