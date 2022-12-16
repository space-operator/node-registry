#[no_mangle]
fn main(numbers: &String) -> Box<String> {
    let mut numbers: Vec<i64> = serde_json::from_str(&numbers).unwrap();

    // bubble sort algorithm
    let mut i = 0;
    while i < numbers.len() {
        let mut j = 0;
        while j < numbers.len() - 1 {
            if numbers[j] > numbers[j + 1] {
                numbers.swap(j, j + 1);
            }
            j += 1;
        }
        i += 1;
    }

    Box::new(serde_json::to_string(&numbers).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bubble_sort() {
        let numbers = Box::new(String::from("[3, 2, 1]"));
        let result = main(&numbers);
        assert_eq!(*result, "[1,2,3]");
    }
}
