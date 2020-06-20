extern crate rpi_cs_core;

use rpi_cs_core::bme280::Bme280;

fn main() {
    let mut bme280 = Bme280::new();
    bme280.init().unwrap();
    let measurements = bme280.measure().unwrap();
    println!("Temperature: {}C", measurements.temperature);
    println!("Pressure: {}hPa", measurements.pressure);
    println!("Humidity: {}%", measurements.humidity);
}