#[no_mangle]
fn main(numbers: &String) -> Box<String> {
    let mut numbers: Vec<i64> = serde_json::from_str(&numbers).unwrap();
    numbers.sort();
    numbers.reverse();
    Box::new(serde_json::to_string(&numbers).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_sort_rev_works() {
        let numbers: Vec<i64> = vec![50, 100000000000, -9223372036854775808, 0];
        let numbers = serde_json::to_string(&numbers).unwrap();
        let sorted: Vec<i64> = vec![100000000000, 50, 0, -9223372036854775808];
        let sorted = serde_json::to_string(&sorted).unwrap();
        assert_eq!(*main(&numbers), sorted);
    }
}
