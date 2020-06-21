use crate::bme280::Bme280;

trait WeatherProvider {
  fn get_temperature(&mut self) -> f32;
  fn get_pressure(&mut self) -> f32;
  fn get_humidity(&mut self) -> f32;
}

pub struct Bme280WeatherProvider {
  bme280: Bme280,
}

impl Bme280WeatherProvider {
  fn new() -> Bme280WeatherProvider {
    let mut bme280 = Bme280::new();
    bme280.init().unwrap();
    Bme280WeatherProvider { bme280 }
  }
}

impl WeatherProvider for Bme280WeatherProvider {
  fn get_temperature(&mut self) -> f32 {
    let measurements = self.bme280.measure().unwrap();
    measurements.temperature
  }

  fn get_pressure(&mut self) -> f32 {
    let measurements = self.bme280.measure().unwrap();
    measurements.pressure
  }

  fn get_humidity(&mut self) -> f32 {
    let measurements = self.bme280.measure().unwrap();
    measurements.humidity
  }
}
