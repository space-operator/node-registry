#[no_mangle]
fn main(numbers: &String) -> Box<String> {
    let mut numbers: Vec<i64> = serde_json::from_str(&numbers).unwrap();

    // quicksort
    numbers = quicksort(numbers);

    Box::new(serde_json::to_string(&numbers).unwrap())
}

fn quicksort(arr: Vec<i64>) -> Vec<i64> {
    if arr.len() <= 1 {
        return arr;
    }
    let pivot = arr[0];
    let mut left: Vec<i64> = Vec::new();
    let mut right: Vec<i64> = Vec::new();
    for i in 1..arr.len() {
        if arr[i] < pivot {
            left.push(arr[i]);
        } else {
            right.push(arr[i]);
        }
    }
    let mut left = quicksort(left);
    let mut right = quicksort(right);
    left.push(pivot);
    left.append(&mut right);
    left
}
