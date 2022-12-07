use space_lib::Request;

#[no_mangle]
fn main(lat: f32, lon: f32) -> Box<String> {
    let response = Request::get("https://api.openweathermap.org/data/2.5/weather")
        .query("lat", lat)
        .query("lon", lon)
        .query("appid", "1b9e9e002f78967f4c2c9a4dfd1a3125")
        .call()
        .unwrap()
        .into_string()
        .unwrap();
    Box::new(response)
}
