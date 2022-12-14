use getrandom::getrandom;

#[no_mangle]
extern fn main(start: u64, end: u64) -> u64 {
    let mut buffer = [0; 8];
    match getrandom(&mut buffer) {
        Ok(_) => u64::from_le_bytes(buffer) % end.saturating_sub(start) + start,
        Err(_) => start,
    }
}
