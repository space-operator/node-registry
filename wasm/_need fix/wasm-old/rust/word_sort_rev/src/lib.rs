//#[no_mangle]
fn main(words: &String) -> Box<String> {
    let mut words: Vec<String> = serde_json::from_str(&words).unwrap();
    words.sort();
    words.reverse();
    Box::new(serde_json::to_string(&words).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_sort_reverse_works() {
        let words = vec![
            "banana".to_string(),
            "apple".to_string(),
            "mango".to_string(),
            "orange".to_string(),
        ];
        let words = serde_json::to_string(&words).unwrap();

        let sorted = vec![
            "orange".to_string(),
            "mango".to_string(),
            "banana".to_string(),
            "apple".to_string(),
        ];
        let sorted = serde_json::to_string(&sorted).unwrap();

        assert_eq!(*main(&words), sorted);
    }
}
