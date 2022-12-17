#[no_mangle]
fn main(array: &String, ordering: &String, algorithm: &String) -> Box<String> {
    serialize_result(sort(array, ordering, algorithm))
}

use std::cmp::Ordering;

use rand::{thread_rng, Rng};
use serde_json::{Number, Value};
use thiserror::Error as ThisError;

use util::*;

#[derive(Clone, Copy)]
enum SortOrdering {
    Ascending,
    Descending,
}

#[derive(Debug)]
enum SortType {
    Default,
    Bubble,
    Insertion,
    Selection,
    Merge,
    Quick,
    Heap,
    Shell,
    Radix,
    Bucket,
    Tim,
    Comb,
    Cycle,
    Strand,
    Bogo,
    Stooge,
    Pigeonhole,
    Cocktail,
}

#[derive(Debug, ThisError)]
enum SortError {
    #[error("first argument can not be deserialized as array")]
    Decode(serde_json::Error),
    #[error("failed to parse \"{0}\" as ordering, valid values are \"asc\" and \"desc\"")]
    Ordering(String),
    #[error("failed to parse \"{0}\" as a Sorttype")]
    SortType(String),
    #[error(transparent)]
    Compare(#[from] CompareError),
}

fn sort(array: &str, ordering: &str, algorithm: &str) -> Result<Vec<Value>, SortError> {
    let mut array: Vec<Value> = serde_json::from_str(array).map_err(SortError::Decode)?;

    let ordering = match ordering {
        "asc" => SortOrdering::Ascending,
        "desc" => SortOrdering::Descending,
        ordering => Err(SortError::Ordering(ordering.to_owned()))?,
    };

    let sort_type = match algorithm {
        "default" => SortType::Default,
        "bubble" => SortType::Bubble,
        "insertion" => SortType::Insertion,
        "selection" => SortType::Selection,
        "merge" => SortType::Merge,
        "quick" => SortType::Quick,
        "heap" => SortType::Heap,
        "shell" => SortType::Shell,
        "radix" => SortType::Radix,
        "bucket" => SortType::Bucket,
        "tim" => SortType::Tim,
        "comb" => SortType::Comb,
        "cycle" => SortType::Cycle,
        "strand" => SortType::Strand,
        "bogo" => SortType::Bogo,
        "stooge" => SortType::Stooge,
        "pigeonhole" => SortType::Pigeonhole,
        "cocktail" => SortType::Cocktail,
        algorithm => Err(SortError::SortType(algorithm.to_owned()))?,
    };

    match sort_type {
        SortType::Default => sort_default(&mut array, ordering),
        SortType::Bubble => sort_bubble(&mut array, ordering),
        SortType::Insertion => sort_insertion(&mut array, ordering),
        SortType::Selection => sort_selection(&mut array, ordering),
        SortType::Merge => sort_merge(&mut array, ordering),
        SortType::Quick => sort_quick(&mut array, ordering),
        SortType::Heap => sort_heap(&mut array, ordering),
        SortType::Shell => sort_shell(&mut array, ordering),
        // SortType::Radix => sort_radix(&mut array, ordering),
        // SortType::Bucket => sort_bucket(&mut array, ordering),
        // SortType::Tim => sort_tim(&mut array, ordering),
        SortType::Comb => sort_comb(&mut array, ordering),
        // SortType::Cycle => sort_cycle(&mut array, ordering),
        SortType::Strand => sort_strand(&mut array, ordering),
        SortType::Bogo => sort_bogo(&mut array, ordering),
        SortType::Stooge => sort_stooge(&mut array, ordering),
        // SortType::Pigeonhole => sort_pigeonhole(&mut array, ordering),
        SortType::Cocktail => sort_cocktail(&mut array, ordering),
        e => Err(SortError::SortType(format!("{:?}", e)))?,
    }
    .map_err(SortError::Compare)?;

    Ok(array)
}

fn sort_default(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
    // sort by default
    let mut err = None;
    match ordering {
        SortOrdering::Ascending => array.sort_by(|lhs, rhs| match compare(lhs, rhs) {
            Ok(ordering) => ordering,
            Err(_) => {
                if err.is_none() {
                    err = Some(());
                }
                Ordering::Less
            }
        }),
        SortOrdering::Descending => array.sort_by(|lhs, rhs| match compare(lhs, rhs) {
            Ok(ordering) => ordering.reverse(),
            Err(_) => {
                if err.is_none() {
                    err = Some(());
                }
                Ordering::Less
            }
        }),
    }

    Ok(())
}

fn sort_bubble(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
    // bubble sort

    match ordering {
        SortOrdering::Ascending => {
            for i in 0..array.len() {
                for j in 0..array.len() - i - 1 {
                    if compare(&array[j], &array[j + 1])? == Ordering::Greater {
                        array.swap(j, j + 1);
                    }
                }
            }
        }
        SortOrdering::Descending => {
            for i in 0..array.len() {
                for j in 0..array.len() - i - 1 {
                    if compare(&array[j], &array[j + 1])? == Ordering::Less {
                        array.swap(j, j + 1);
                    }
                }
            }
        }
    }

    Ok(())
}

fn sort_insertion(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
    // insertion sort

    match ordering {
        SortOrdering::Ascending => {
            for i in 1..array.len() {
                let mut j = i;
                while j > 0 && compare(&array[j - 1], &array[j])? == Ordering::Greater {
                    array.swap(j - 1, j);
                    j -= 1;
                }
            }
        }
        SortOrdering::Descending => {
            for i in 1..array.len() {
                let mut j = i;
                while j > 0 && compare(&array[j - 1], &array[j])? == Ordering::Less {
                    array.swap(j - 1, j);
                    j -= 1;
                }
            }
        }
    }

    Ok(())
}

fn sort_selection(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
    // selection sort

    match ordering {
        SortOrdering::Ascending => {
            for i in 0..array.len() {
                let mut min = i;
                for j in i + 1..array.len() {
                    if compare(&array[j], &array[min])? == Ordering::Less {
                        min = j;
                    }
                }
                array.swap(i, min);
            }
        }
        SortOrdering::Descending => {
            for i in 0..array.len() {
                let mut max = i;
                for j in i + 1..array.len() {
                    if compare(&array[j], &array[max])? == Ordering::Greater {
                        max = j;
                    }
                }
                array.swap(i, max);
            }
        }
    }

    Ok(())
}

fn sort_merge(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
    // merge sort

    fn merge(
        array: &mut [Value],
        ordering: SortOrdering,
        left: usize,
        mid: usize,
        right: usize,
    ) -> Result<(), CompareError> {
        let left_array = array[left..mid + 1].to_vec();
        let right_array = array[mid + 1..right + 1].to_vec();

        let mut i = 0;
        let mut j = 0;
        let mut k = left;

        while i < left_array.len() && j < right_array.len() {
            match ordering {
                SortOrdering::Ascending => {
                    if compare(&left_array[i], &right_array[j])? == Ordering::Less {
                        array[k] = left_array[i].clone();
                        i += 1;
                    } else {
                        array[k] = right_array[j].clone();
                        j += 1;
                    }
                }
                SortOrdering::Descending => {
                    if compare(&left_array[i], &right_array[j])? == Ordering::Greater {
                        array[k] = left_array[i].clone();
                        i += 1;
                    } else {
                        array[k] = right_array[j].clone();
                        j += 1;
                    }
                }
            }
            k += 1;
        }

        while i < left_array.len() {
            array[k] = left_array[i].clone();
            i += 1;
            k += 1;
        }

        while j < right_array.len() {
            array[k] = right_array[j].clone();
            j += 1;
            k += 1;
        }

        Ok(())
    }

    fn merge_sort(
        array: &mut [Value],
        ordering: SortOrdering,
        left: usize,
        right: usize,
    ) -> Result<(), CompareError> {
        if left < right {
            let mid = left + (right - left) / 2;
            merge_sort(array, ordering, left, mid)?;
            merge_sort(array, ordering, mid + 1, right)?;
            merge(array, ordering, left, mid, right)?;
        }

        Ok(())
    }

    merge_sort(array, ordering, 0, array.len() - 1)
}

fn sort_quick(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
    // simple quick sort (not randomized)

    fn partition(
        array: &mut [Value],
        ordering: SortOrdering,
        left: usize,
        right: usize,
    ) -> Result<usize, CompareError> {
        let pivot = array[right].clone();
        let mut i = left;

        for j in left..right {
            match ordering {
                SortOrdering::Ascending => {
                    if compare(&array[j], &pivot)? == Ordering::Less {
                        array.swap(i, j);
                        i += 1;
                    }
                }
                SortOrdering::Descending => {
                    if compare(&array[j], &pivot)? == Ordering::Greater {
                        array.swap(i, j);
                        i += 1;
                    }
                }
            }
        }

        array.swap(i, right);

        Ok(i)
    }

    fn quick_sort(
        array: &mut [Value],
        ordering: SortOrdering,
        left: usize,
        right: usize,
    ) -> Result<(), CompareError> {
        if left < right {
            let pivot = partition(array, ordering, left, right)?;

            if pivot > 0 {
                quick_sort(array, ordering, left, pivot - 1)?;
            }

            quick_sort(array, ordering, pivot + 1, right)?;
        }

        Ok(())
    }

    quick_sort(array, ordering, 0, array.len() - 1)
}

fn sort_heap(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
    // heap sort

    fn heapify(
        array: &mut [Value],
        ordering: SortOrdering,
        length: usize,
        index: usize,
    ) -> Result<(), CompareError> {
        let compare = |a: &Value, b: &Value| match ordering {
            SortOrdering::Ascending => compare(a, b),
            SortOrdering::Descending => compare(b, a),
        };

        let mut largest = index;
        let left = 2 * index + 1;
        let right = 2 * index + 2;

        if left < length && compare(&array[left], &array[largest])? == Ordering::Greater {
            largest = left;
        }

        if right < length && compare(&array[right], &array[largest])? == Ordering::Greater {
            largest = right;
        }

        if largest != index {
            array.swap(index, largest);
            heapify(array, ordering, length, largest)?;
        }

        Ok(())
    }

    fn build_heap(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
        let length = array.len();
        for i in (0..length / 2).rev() {
            heapify(array, ordering, length, i)?;
        }

        Ok(())
    }

    build_heap(array, ordering)?;

    for i in (1..array.len()).rev() {
        array.swap(0, i);
        heapify(array, ordering, i, 0)?;
    }

    Ok(())
}

fn sort_shell(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
    // shell sort

    let compare = |a: &Value, b: &Value| match ordering {
        SortOrdering::Ascending => compare(a, b),
        SortOrdering::Descending => compare(b, a),
    };

    let length = array.len();
    let mut gap = length / 2;

    while gap > 0 {
        for i in gap..length {
            let mut j = i;
            while j >= gap && compare(&array[j - gap], &array[j])? == Ordering::Greater {
                array.swap(j, j - gap);
                j -= gap;
            }
        }

        gap /= 2;
    }

    Ok(())
}

// fn sort_radix(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
//     // radix sort

//     fn get_max(array: &[Value]) -> Result<u32, CompareError> {
//         let mut max = Value::Null;

//         for value in array {
//             // handle with compare function
//             if let Value::Number(number) = value {
//                 if let Number::U32(number) = number {
//                     if compare(&max, &Value::Number(Number::U32(*number)))? == Ordering::Less {
//                         max = Value::Number(Number::U32(*number));
//                     }
//                 }
//             }
//         }

//         Ok(max)
//     }

//     fn count_sort(
//         array: &mut [Value],
//         ordering: SortOrdering,
//         exp: u64,
//     ) -> Result<(), CompareError> {
//         let mut output = vec![Value::Null; array.len()];
//         let mut count = vec![0; 10];

//         for i in 0..array.len() {
//             if let Value::Number(number) = &array[i] {
//                 if let Some(number) = number.as_u64() {
//                     count[((number / exp) % 10) as usize] += 1;
//                 }
//             }
//         }

//         for i in 1..10 {
//             count[i] += count[i - 1];
//         }

//         let mut i = array.len();
//         while i > 0 {
//             i -= 1;
//             if let Value::Number(number) = &array[i] {
//                 if let Some(number) = number.as_u64() {
//                     output[count[((number / exp) % 10) as usize] - 1] = array[i];
//                     count[((number / exp) % 10) as usize] -= 1;
//                 }
//             }
//         }

//         for i in 0..array.len() {
//             array[i] = output[i];
//         }

//         Ok(())
//     }

//     let max = get_max(array)?;

//     let mut exp = 1;
//     while max / exp > 0 {
//         count_sort(array, ordering, exp)?;
//         exp *= 10;
//     }

//     Ok(())
// }

// fn sort_bucket(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
//     // bucket sort

//     fn get_max(array: &[Value]) -> Result<u32, CompareError> {
//         let mut max = 0;

//         for value in array {
//             if let Value::Number(number) = value {
//                 if let Number::U32(number) = number {
//                     if *number > max {
//                         max = *number;
//                     }
//                 }
//             }
//         }

//         Ok(max)
//     }

//     fn get_min(array: &[Value]) -> Result<u32, CompareError> {
//         let mut min = u32::MAX;

//         for value in array {
//             if let Value::Number(number) = value {
//                 if let Number::U32(number) = number {
//                     if *number < min {
//                         min = *number;
//                     }
//                 }
//             }
//         }

//         Ok(min)
//     }

//     let max = get_max(array)?;
//     let min = get_min(array)?;

//     let mut buckets = vec![Vec::new(); array.len()];

//     for value in array {
//         if let Value::Number(number) = value {
//             if let Number(number) = number {
//                 let index = ((number - min) * (array.len() - 1) as u32 / (max - min)) as usize;
//                 buckets[index].push(value);
//             }
//         }
//     }

//     for bucket in buckets.iter_mut() {
//         bucket.sort_by(|lhs, rhs| compare(lhs, rhs).unwrap());
//     }

//     let mut index = 0;
//     for bucket in buckets {
//         for value in bucket {
//             array[index] = value;
//             index += 1;
//         }
//     }

//     Ok(())
// }

fn sort_comb(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
    let compare = |a: &Value, b: &Value| match ordering {
        SortOrdering::Ascending => compare(a, b),
        SortOrdering::Descending => compare(b, a),
    };

    // comb sort

    let mut gap = array.len();
    let shrink = 1.3;

    let mut swapped = true;
    while gap > 1 || swapped {
        gap = (gap as f64 / shrink).floor() as usize;

        let mut i = 0;
        swapped = false;
        while i + gap < array.len() {
            if compare(&array[i], &array[i + gap])? == Ordering::Greater {
                array.swap(i, i + gap);
                swapped = true;
            }

            i += 1;
        }
    }

    Ok(())
}

fn sort_cycle(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
    // cycle sort

    let mut writes = 0;

    let compare = |a: &Value, b: &Value| match ordering {
        SortOrdering::Ascending => compare(a, b),
        SortOrdering::Descending => compare(b, a),
    };

    let mut cycle_start = 0;
    while writes < array.len() {
        let item = array[cycle_start].clone();

        let mut pos = cycle_start;
        for i in cycle_start + 1..array.len() {
            if compare(&array[i], &item)? == Ordering::Less {
                pos += 1;
            }
        }

        if pos == cycle_start {
            cycle_start += 1;
            continue;
        }

        while compare(&item, &array[pos])? == Ordering::Equal {
            pos += 1;
        }

        array.swap(pos, cycle_start);

        writes += 1;

        while pos != cycle_start {
            pos = cycle_start;

            for i in cycle_start + 1..array.len() {
                if compare(&array[i], &item)? == Ordering::Less {
                    pos += 1;
                }
            }

            while compare(&item, &array[pos])? == Ordering::Equal {
                pos += 1;
            }

            array.swap(pos, cycle_start);

            writes += 1;
        }

        cycle_start += 1;
    }

    Ok(())
}

fn sort_strand(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
    // strand sort

    let compare = |a: &Value, b: &Value| match ordering {
        SortOrdering::Ascending => compare(a, b),
        SortOrdering::Descending => compare(b, a),
    };

    let mut i = 1;
    while i < array.len() {
        if compare(&array[i - 1], &array[i])? == Ordering::Greater {
            array.swap(i - 1, i);
            if i > 1 {
                i -= 1;
            }
        } else {
            i += 1;
        }
    }

    Ok(())
}

fn sort_bogo(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
    fn is_sorted(array: &[Value], ordering: SortOrdering) -> Result<bool, CompareError> {
        let compare = |a: &Value, b: &Value| match ordering {
            SortOrdering::Ascending => compare(a, b),
            SortOrdering::Descending => compare(b, a),
        };

        for i in 1..array.len() {
            if compare(&array[i - 1], &array[i])? == Ordering::Greater {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn shuffle(array: &mut [Value]) {
        for i in 0..array.len() {
            let j = thread_rng().gen_range(0..array.len());
            array.swap(i, j);
        }
    }

    // bogo sort

    while !is_sorted(array, ordering)? {
        shuffle(array);
    }

    Ok(())
}

fn sort_stooge(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
    // stooge sort

    fn stooge(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
        let compare = |a: &Value, b: &Value| match ordering {
            SortOrdering::Ascending => compare(a, b),
            SortOrdering::Descending => compare(b, a),
        };

        if array.len() < 2 {
            return Ok(());
        }

        if compare(&array[0], &array[array.len() - 1])? == Ordering::Greater {
            array.swap(0, array.len() - 1);
        }
        let len = array.len();
        if len > 2 {
            let t = array.len() / 3;
            stooge(&mut array[..len - t], ordering)?;
            stooge(&mut array[t..], ordering)?;
            stooge(&mut array[..len - t], ordering)?;
        }

        Ok(())
    }

    stooge(array, ordering)
}

fn sort_cocktail(array: &mut [Value], ordering: SortOrdering) -> Result<(), CompareError> {
    // cocktail sort

    let compare = |a: &Value, b: &Value| match ordering {
        SortOrdering::Ascending => compare(a, b),
        SortOrdering::Descending => compare(b, a),
    };

    let mut swapped = true;
    let mut start = 0;
    let mut end = array.len();

    while swapped {
        swapped = false;

        for i in start..end - 1 {
            if compare(&array[i], &array[i + 1])? == Ordering::Greater {
                array.swap(i, i + 1);
                swapped = true;
            }
        }

        if !swapped {
            break;
        }

        swapped = false;
        end -= 1;

        for i in (start..end - 1).rev() {
            if compare(&array[i], &array[i + 1])? == Ordering::Greater {
                array.swap(i, i + 1);
                swapped = true;
            }
        }

        start += 1;
    }

    Ok(())
}

#[derive(Debug, ThisError)]
enum CompareError {
    #[error("failed to compare mixed types: \"{0}\" and \"{1}\"")]
    Mixed(Value, Value),
    #[error("failed to compare numbers: \"{0}\" and \"{1}\"")]
    Number(Number, Number),
    #[error("array with arrays can not be sorted")]
    Array,
    #[error("array with objects can not be sorted")]
    Object,
}

fn compare(lhs: &Value, rhs: &Value) -> Result<Ordering, CompareError> {
    // able to compare strings with numbers and booleans
    match (lhs, rhs) {
        (Value::String(lhs), Value::String(rhs)) => Ok(lhs.cmp(rhs)),
        (Value::String(lhs), Value::Number(rhs)) => Ok(lhs.cmp(&rhs.to_string())),
        (Value::String(lhs), Value::Bool(rhs)) => Ok(lhs.cmp(&rhs.to_string())),
        (Value::Number(lhs), Value::String(rhs)) => Ok(lhs.to_string().cmp(rhs)),
        (Value::Number(lhs), Value::Number(rhs)) => {
            // try as i64 then u64 then f64
            if let (Some(l), Some(r)) = (lhs.as_i64(), rhs.as_i64()) {
                Ok(l.cmp(&r))
            } else if let (Some(l), Some(r)) = (lhs.as_u64(), rhs.as_u64()) {
                Ok(l.cmp(&r))
            } else if let (Some(l), Some(r)) = (lhs.as_f64(), rhs.as_f64()) {
                Ok(l.partial_cmp(&r)
                    .ok_or_else(|| CompareError::Number(lhs.clone(), rhs.clone()))?)
            } else {
                Err(CompareError::Number(lhs.clone(), rhs.clone()))
            }
        }
        (Value::Number(lhs), Value::Bool(rhs)) => Ok(lhs.to_string().cmp(&rhs.to_string())),
        (Value::Bool(lhs), Value::String(rhs)) => Ok(lhs.to_string().cmp(rhs)),
        (Value::Bool(lhs), Value::Number(rhs)) => Ok(lhs.to_string().cmp(&rhs.to_string())),
        (Value::Bool(lhs), Value::Bool(rhs)) => Ok(lhs.cmp(rhs)),
        (Value::Array(_), _) => Err(CompareError::Array),
        (Value::Object(_), _) => Err(CompareError::Object),
        (Value::Null, _) => Ok(Ordering::Less),
        (_, Value::Null) => Ok(Ordering::Greater),
        _ => Err(CompareError::Mixed(lhs.clone(), rhs.clone())),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::sort;

    #[test]
    fn sorts() {
        // test each sort algorithm
        let mut scrambled = json!(vec![
            json!(6),
            json!(8),
            json!(2),
            json!(0),
            json!(3),
            json!(4),
            json!(1),
            json!(-1),
            json!(2),
            json!(6)
        ])
        .to_string();

        let sorted = vec![
            json!(-1),
            json!(0),
            json!(1),
            json!(2),
            json!(2),
            json!(3),
            json!(4),
            json!(6),
            json!(6),
            json!(8),
        ];

        let reversed = vec![
            json!(8),
            json!(6),
            json!(6),
            json!(4),
            json!(3),
            json!(2),
            json!(2),
            json!(1),
            json!(0),
            json!(-1),
        ];

        let result = sort(&mut scrambled, "asc", "default").unwrap();
        assert_eq!(result, sorted);
        println!("success: \"default\"");

        let result = sort(&mut scrambled, "asc", "bubble").unwrap();
        assert_eq!(result, sorted);
        println!("success: \"bubble\"");

        let result = sort(&mut scrambled, "asc", "insertion").unwrap();
        assert_eq!(result, sorted);
        println!("success: \"insertion\"");

        let result = sort(&mut scrambled, "asc", "selection").unwrap();
        assert_eq!(result, sorted);
        println!("success: \"selection\"");

        let result = sort(&mut scrambled, "asc", "merge").unwrap();
        assert_eq!(result, sorted);
        println!("success: \"merge\"");

        let result = sort(&mut scrambled, "asc", "quick").unwrap();
        assert_eq!(result, sorted);
        println!("success: \"quick\"");

        let result = sort(&mut scrambled, "asc", "heap").unwrap();
        assert_eq!(result, sorted);
        println!("success: \"heap\"");

        let result = sort(&mut scrambled, "asc", "shell").unwrap();
        assert_eq!(result, sorted);
        println!("success: \"shell\"");

        let result = sort(&mut scrambled, "asc", "comb").unwrap();
        assert_eq!(result, sorted);
        println!("success: \"comb\"");

        // let result = sort(&mut scrambled, "asc", "cycle").unwrap();
        // assert_eq!(result, sorted);
        // println!("success: \"cycle\"");

        let result = sort(&mut scrambled, "asc", "strand").unwrap();
        assert_eq!(result, sorted);
        println!("success: \"strand\"");

        // we wont test bogo sort because it is random
        // let result = sort(&mut scrambled, "asc", "bogo").unwrap();
        // assert_eq!(result, sorted);

        let result = sort(&mut scrambled, "asc", "stooge").unwrap();
        assert_eq!(result, sorted);
        println!("success: \"stooge\"");

        let result = sort(&mut scrambled, "asc", "cocktail").unwrap();
        assert_eq!(result, sorted);
        println!("success: \"cocktail\"");

        let result = sort(&mut scrambled, "desc", "default").unwrap();
        assert_eq!(result, reversed);
        println!("success: \"default\"");

        let result = sort(&mut scrambled, "desc", "bubble").unwrap();
        assert_eq!(result, reversed);
        println!("success: \"bubble\"");

        let result = sort(&mut scrambled, "desc", "insertion").unwrap();
        assert_eq!(result, reversed);
        println!("success: \"insertion\"");

        let result = sort(&mut scrambled, "desc", "selection").unwrap();
        assert_eq!(result, reversed);
        println!("success: \"selection\"");

        let result = sort(&mut scrambled, "desc", "merge").unwrap();
        assert_eq!(result, reversed);
        println!("success: \"merge\"");

        let result = sort(&mut scrambled, "desc", "quick").unwrap();
        assert_eq!(result, reversed);
        println!("success: \"quick\"");

        let result = sort(&mut scrambled, "desc", "heap").unwrap();
        assert_eq!(result, reversed);
        println!("success: \"heap\"");

        let result = sort(&mut scrambled, "desc", "shell").unwrap();
        assert_eq!(result, reversed);
        println!("success: \"shell\"");

        let result = sort(&mut scrambled, "desc", "comb").unwrap();
        assert_eq!(result, reversed);
        println!("success: \"comb\"");

        // let result = sort(&mut scrambled, "desc", "cycle").unwrap();
        // assert_eq!(result, reversed);
        // println!("success: \"cycle\"");

        let result = sort(&mut scrambled, "desc", "strand").unwrap();
        assert_eq!(result, reversed);
        println!("success: \"strand\"");

        // let result = sort(&mut scrambled, "desc", "bogo").unwrap();
        // assert_eq!(result, reversed);

        let result = sort(&mut scrambled, "desc", "stooge").unwrap();
        assert_eq!(result, reversed);
        println!("success: \"stooge\"");

        let result = sort(&mut scrambled, "desc", "cocktail").unwrap();
        assert_eq!(result, reversed);
        println!("success: \"cocktail\"");

        // test invalid sort type
        let result = sort(&mut scrambled, "asc", "invalid");
        assert!(result.is_err());
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
