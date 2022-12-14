use uuid::Uuid;

#[no_mangle]
extern "C" fn main() -> Box<String> {
    Box::new(Uuid::new_v4().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_generated_with_correct_length_works() {
        let uuid = main();
        assert_eq!(uuid.len(), 36);
    }
}
