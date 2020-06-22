extern crate dotenv;
extern crate rpi_cs_core;

use dotenv::dotenv;
use rpi_cs_core::bme280::Bme280;
use rpi_cs_core::api;

fn main() {
    dotenv().ok();
    api::start().launch();
}