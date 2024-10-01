pub struct WeatherSummary {
    temperature: f32,
}

pub fn get_weather_forecast() -> WeatherSummary {
    WeatherSummary {temperature: 3.}
}
pub fn build_weather_digest(digest: &mut String, weather: WeatherSummary) {

}