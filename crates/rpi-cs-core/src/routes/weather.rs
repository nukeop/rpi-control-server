use rocket::http::Status;
use rocket::State;
use rocket_contrib::json::Json;

use crate::bme280::{Bme280, Measurements};
use crate::debug::DebugState;

pub trait WeatherApiTrait {
  fn weather(&mut self) -> Result<Json<Measurements>, Status>;
}

pub struct Bme280WeatherApi {
  bme280: Bme280,
}

impl Bme280WeatherApi {
  pub fn new() -> Bme280WeatherApi {
    let mut bme280 = Bme280::new();
    bme280.init().unwrap();
    Bme280WeatherApi { bme280 }
  }
}

impl WeatherApiTrait for Bme280WeatherApi {
  fn weather(&mut self) -> Result<Json<Measurements>, Status> {
    let measurements = self.bme280.measure();
    match measurements {
      Ok(data) => Ok(Json(data)),
      Err(_) => Err(Status::new(500, "Invalid data")),
    }
  }
}

pub struct MockWeatherApi {}

impl WeatherApiTrait for MockWeatherApi {
  fn weather(&mut self) -> Result<Json<Measurements>, Status> {
    Ok(Json(Measurements {
      temperature: 1.0,
      pressure: 1.0,
      humidity: 1.0,
    }))
  }
}

#[get("/weather")]
pub fn weather(debugState: State<DebugState>) -> Result<Json<Measurements>, Status> {
  if debugState.debug {
    let mut weatherApi = MockWeatherApi {};
    weatherApi.weather()
  } else {
    let mut weatherApi = Bme280WeatherApi::new();
    weatherApi.weather()
  }
}
