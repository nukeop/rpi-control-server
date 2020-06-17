use rppal::i2c::I2c;

static i2c_address: u16 = 0x76;

pub struct Bme280 {
pub i2c: I2c
}

impl Bme280 {
  pub fn new() -> Bme280 {
    Bme280 {
      i2c: I2c::new().unwrap()
    }
  }

  pub fn init(&mut self) {
    self.i2c.set_slave_address(i2c_address).unwrap();
  }
}