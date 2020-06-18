use rppal::i2c::I2c;

const I2C_ADDRESS: u16 = 0x76;

const BME280_CHIP_ID: u8 = 0x60;
const BME280_CHIP_ID_ADDR: u8 = 0xD0;

const BME280_RESET_ADDR: u8 = 0xE0;
const BME280_SOFT_RESET_CMD: u8 = 0xB6;

const BME280_P_T_CALIB_DATA_ADDR: u8 = 0x88;
const BME280_P_T_CALIB_DATA_LEN: usize = 26;

const BME280_H_CALIB_DATA_ADDR: u8 = 0xE1;
const BME280_H_CALIB_DATA_LEN: usize = 7;

macro_rules! concat_bytes {
  ($msb:expr, $lsb:expr) => {
    (($msb as u16) << 8) | ($lsb as u16)
  };
}

#[derive(Debug)]
pub enum Error {
  /// Failed to compensate a raw measurement
  CompensationFailed,
  /// I²C bus error
  I2C,
  /// Failed to parse sensor data
  InvalidData,
  /// No calibration data is available (probably forgot to call or check BME280::init for failure)
  NoCalibrationData,
  /// Chip ID doesn't match expected value
  UnsupportedChip,
}

#[derive(Debug)]
pub struct CalibrationData {
  dig_t1: u16,
  dig_t2: i16,
  dig_t3: i16,
  dig_p1: u16,
  dig_p2: i16,
  dig_p3: i16,
  dig_p4: i16,
  dig_p5: i16,
  dig_p6: i16,
  dig_p7: i16,
  dig_p8: i16,
  dig_p9: i16,
  dig_h1: u8,
  dig_h2: i16,
  dig_h3: u8,
  dig_h4: i16,
  dig_h5: i16,
  dig_h6: i8,
  t_fine: i32,
}

pub struct Bme280 {
  pub i2c: I2c,
  pub calib_data: Option<CalibrationData>,
}

fn parse_calib_data(
  pt_data: &[u8; BME280_P_T_CALIB_DATA_LEN],
  h_data: &[u8; BME280_H_CALIB_DATA_LEN],
) -> CalibrationData {
  let dig_t1 = concat_bytes!(pt_data[1], pt_data[0]);
  let dig_t2 = concat_bytes!(pt_data[3], pt_data[2]) as i16;
  let dig_t3 = concat_bytes!(pt_data[5], pt_data[4]) as i16;
  let dig_p1 = concat_bytes!(pt_data[7], pt_data[6]);
  let dig_p2 = concat_bytes!(pt_data[9], pt_data[8]) as i16;
  let dig_p3 = concat_bytes!(pt_data[11], pt_data[10]) as i16;
  let dig_p4 = concat_bytes!(pt_data[13], pt_data[12]) as i16;
  let dig_p5 = concat_bytes!(pt_data[15], pt_data[14]) as i16;
  let dig_p6 = concat_bytes!(pt_data[17], pt_data[16]) as i16;
  let dig_p7 = concat_bytes!(pt_data[19], pt_data[18]) as i16;
  let dig_p8 = concat_bytes!(pt_data[21], pt_data[20]) as i16;
  let dig_p9 = concat_bytes!(pt_data[23], pt_data[22]) as i16;
  let dig_h1 = pt_data[25];
  let dig_h2 = concat_bytes!(h_data[1], h_data[0]) as i16;
  let dig_h3 = h_data[2];
  let dig_h4 = (h_data[3] as i16 * 16) | ((h_data[4] as i16) & 0x0F);
  let dig_h5 = (h_data[5] as i16 * 16) | ((h_data[4] as i16) >> 4);
  let dig_h6 = h_data[6] as i8;

  CalibrationData {
    dig_t1,
    dig_t2,
    dig_t3,
    dig_p1,
    dig_p2,
    dig_p3,
    dig_p4,
    dig_p5,
    dig_p6,
    dig_p7,
    dig_p8,
    dig_p9,
    dig_h1,
    dig_h2,
    dig_h3,
    dig_h4,
    dig_h5,
    dig_h6,
    t_fine: 0,
  }
}

impl Bme280 {
  pub fn new() -> Bme280 {
    Bme280 {
      i2c: I2c::new().unwrap(),
      calib_data: None,
    }
  }

  pub fn init(&mut self) -> Result<(), Error> {
    self.verify_chip_id()?;
    self.soft_reset().unwrap();
    self.calibrate()?;
    self.setup();

    Ok(())
  }

  fn verify_chip_id(&mut self) -> Result<(), Error> {
    let chip_id = self.read_register(BME280_CHIP_ID_ADDR)?;
    if chip_id == BME280_CHIP_ID {
      Ok(())
    } else {
      Err(Error::UnsupportedChip)
    }
  }

  fn soft_reset(&mut self) -> Result<(), rppal::i2c::Error> {
    self.write_reg(BME280_RESET_ADDR, BME280_SOFT_RESET_CMD)?;
    Ok(())
  }

  pub fn setup(&mut self) {
    self.i2c.set_slave_address(I2C_ADDRESS).unwrap();

    let osrs_t = 1; // Temperature oversampling x 1
    let osrs_p = 1; // Pressure oversampling x 1
    let osrs_h = 1; // Humidity oversampling x 1
    let mode = 3; // Normal mode
    let t_sb = 5; // Tstandby 1000ms
    let filter = 0; // Filter off
    let spi3w_en = 0; // 3-wire SPI Disable

    let ctrl_meas_reg = (osrs_t << 5) | (osrs_p << 2) | mode;
    let config_reg = (t_sb << 5) | (filter << 2) | spi3w_en;
    let ctrl_hum_reg = osrs_h;

    self.write_reg(0xF2, ctrl_hum_reg).unwrap();
    self.write_reg(0xF4, ctrl_meas_reg).unwrap();
    self.write_reg(0xF5, config_reg).unwrap();
  }

  fn read_register(&mut self, register: u8) -> Result<u8, Error> {
    let mut data: [u8; 1] = [0];
    let result = self.i2c.write_read(&[register], &mut data);
    match result {
      Ok(_) => Ok(data[0]),
      Err(_) => Err(Error::I2C),
    }
  }

  pub fn read_pt_calib_data(
    &mut self,
    register: u8,
  ) -> Result<[u8; BME280_P_T_CALIB_DATA_LEN], Error> {
    let mut data: [u8; BME280_P_T_CALIB_DATA_LEN] = [0; BME280_P_T_CALIB_DATA_LEN];
    let result = self.i2c.write_read(&[register], &mut data);
    match result {
      Ok(_) => Ok(data),
      Err(_) => Err(Error::I2C),
    }
  }

  fn read_h_calib_data(&mut self, register: u8) -> Result<[u8; BME280_H_CALIB_DATA_LEN], Error> {
    let mut data: [u8; BME280_H_CALIB_DATA_LEN] = [0; BME280_H_CALIB_DATA_LEN];
    let result = self.i2c.write_read(&[register], &mut data);
    match result {
      Ok(_) => Ok(data),
      Err(_) => Err(Error::I2C),
    }
  }

  pub fn calibrate(&mut self) -> Result<(), Error> {
    let pt_calib_data = self.read_pt_calib_data(BME280_P_T_CALIB_DATA_ADDR)?;
    let h_calib_data = self.read_h_calib_data(BME280_H_CALIB_DATA_ADDR)?;
    self.calib_data = Some(parse_calib_data(&pt_calib_data, &h_calib_data));
    Ok(())
  }

  pub fn write_reg(&mut self, register: u8, data: u8) -> Result<(), rppal::i2c::Error> {
    self.i2c.smbus_write_byte(register, data)
  }
}
