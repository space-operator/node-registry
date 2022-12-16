#[no_mangle]
fn main(numbers: &String) -> Box<String> {
    let mut numbers: Vec<i64> = serde_json::from_str(&numbers).unwrap();

    // insertion sort
    let mut i = 1;
    while i < numbers.len() {
        let mut j = i;
        while j > 0 && numbers[j - 1] > numbers[j] {
            numbers.swap(j - 1, j);
            j -= 1;
        }
        i += 1;
    }

    Box::new(serde_json::to_string(&numbers).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insertion_sort() {
        let numbers = Box::new(String::from("[3, 2, 1]"));
        let result = main(&numbers);
        assert_eq!(*result, "[1,2,3]");
    }
}
