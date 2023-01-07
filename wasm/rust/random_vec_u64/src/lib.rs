use rand::Rng;

#[no_mangle]
fn main(min: u64, max: u64, len: u64) -> Box<String> {
    let mut rng = rand::thread_rng();
    let mut vec = Vec::new();

    for _ in 0..len {
        vec.push(rng.gen_range(min..max));
    }

    Box::new(serde_json::to_string(&vec).unwrap())
}
