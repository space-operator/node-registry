use space_lib::Request;

#[no_mangle]
fn main(lat: f32, lon: f32) -> Box<String> {
    let response = Request::get("https://api.openweathermap.org/data/2.5/weather")
        .query("lat", lat)
        .query("lon", lon)
        .query("appid", "7f5a1fda2803b4fd2577113493da20cc")
        .call()
        .unwrap()
        .into_string()
        .unwrap();
    Box::new(response)
}
