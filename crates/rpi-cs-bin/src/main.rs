extern crate rpi_cs_core;

use rpi_cs_core::bme280::Bme280;

fn main() {
    let mut bme280 = Bme280::new();
    bme280.setup();
    bme280.calibrate().unwrap();
    println!("{:?}", bme280.calib_data);
}