// Parts of code copied and adapted from https://github.com/uber-foo/bme280-rs 

use rppal::i2c::I2c;

const I2C_ADDRESS: u16 = 0x76;

const BME280_CHIP_ID: u8 = 0x60;
const BME280_CHIP_ID_ADDR: u8 = 0xD0;

const BME280_DATA_ADDR: u8 = 0xF7;
const BME280_P_T_H_DATA_LEN: usize = 8;

const BME280_P_T_CALIB_DATA_ADDR: u8 = 0x88;
const BME280_P_T_CALIB_DATA_LEN: usize = 24;

const BME280_H_CALIB_DATA_ADDR: u8 = 0xE1;
const BME280_H_CALIB_DATA_LEN: usize = 7;

const BME280_TEMP_MIN: f32 = -40.0;
const BME280_TEMP_MAX: f32 = 85.0;

const BME280_PRESSURE_MIN: f32 = 30000.0;
const BME280_PRESSURE_MAX: f32 = 110000.0;

const BME280_HUMIDITY_MIN: f32 = 0.0;
const BME280_HUMIDITY_MAX: f32 = 100.0;

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

#[derive(Debug, Copy, Clone)]
pub enum SensorMode {
  Sleep,
  Forced,
  Normal,
}

#[derive(Debug)]
pub struct Measurements {
  pub temperature: f32,
  pub pressure: f32,
  pub humidity: f32,
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
  
  let dig_h1 = pt_data[12];
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


impl Measurements {
  fn parse(
      data: [u8; BME280_P_T_H_DATA_LEN],
      calibration: &mut CalibrationData,
  ) -> Result<Self, Error> {
      let data_msb: u32 = (data[0] as u32) << 12;
      let data_lsb: u32 = (data[1] as u32) << 4;
      let data_xlsb: u32 = (data[2] as u32) >> 4;
      let pressure = data_msb | data_lsb | data_xlsb;

      let data_msb: u32 = (data[3] as u32) << 12;
      let data_lsb: u32 = (data[4] as u32) << 4;
      let data_xlsb: u32 = (data[5] as u32) >> 4;
      let temperature = data_msb | data_lsb | data_xlsb;

      let data_msb: u32 = (data[6] as u32) << 8;
      let data_lsb: u32 = data[7] as u32;
      let humidity = data_msb | data_lsb;

      let temperature = Measurements::compensate_temperature(temperature, calibration)?;
      let pressure = Measurements::compensate_pressure(pressure, calibration)?;
      let humidity = Measurements::compensate_humidity(humidity, calibration)?;

      Ok(Measurements {
          temperature,
          pressure,
          humidity
      })
  }

  fn compensate_temperature(
    uncompensated: u32,
    calibration: &mut CalibrationData,
) -> Result<f32, Error> {
    let var1: f32 = uncompensated as f32 / 16384.0 - calibration.dig_t1 as f32 / 1024.0;
    let var1 = var1 * calibration.dig_t2 as f32;
    let var2 = uncompensated as f32 / 131072.0 - calibration.dig_t1 as f32 / 8192.0;
    let var2 = var2 * var2 * calibration.dig_t3 as f32;

    calibration.t_fine = (var1 + var2) as i32;

    let temperature = (var1 + var2) / 5120.0;
    let temperature = if temperature < BME280_TEMP_MIN {
        BME280_TEMP_MIN
    } else if temperature > BME280_TEMP_MAX {
        BME280_TEMP_MAX
    } else {
        temperature
    };
    Ok(temperature)
}

fn compensate_pressure(
    uncompensated: u32,
    calibration: &mut CalibrationData,
) -> Result<f32, Error> {
    let var1: f32 = calibration.t_fine as f32 / 2.0 - 64000.0;
    let var2: f32 = var1 * var1 * calibration.dig_p6 as f32 / 32768.0;
    let var2: f32 = var2 + var1 * calibration.dig_p5 as f32 * 2.0;
    let var2: f32 = var2 / 4.0 + calibration.dig_p4 as f32 * 65536.0;
    let var3: f32 = calibration.dig_p3 as f32 * var1 * var1 / 524288.0;
    let var1: f32 = (var3 + calibration.dig_p2 as f32 * var1) / 524288.0;
    let var1: f32 = (1.0 + var1 / 32768.0) * calibration.dig_p1 as f32;

    let pressure = if var1 > 0.0 {
        let pressure: f32 = 1048576.0 - uncompensated as f32;
        let mut pressure: f32 = (pressure - (var2 / 4096.0)) * 3125.0;
        if pressure < 2147483648.0 {
          pressure = (pressure * 2.0) / var1;
        } else {
          pressure = (pressure / var1) * 2.0;
        }

        let var1: f32 = (calibration.dig_p9 as f32 * (((pressure/8.0) * (pressure/8.0)) / 8192.0)) / 4096.0;
        let var2: f32 = pressure * calibration.dig_p8 as f32 / 32768.0;
        let pressure: f32 = pressure + (var1 + var2 + calibration.dig_p7 as f32) / 16.0;
        if pressure < BME280_PRESSURE_MIN {
            BME280_PRESSURE_MIN
        } else if pressure > BME280_PRESSURE_MAX {
            BME280_PRESSURE_MAX
        } else {
            pressure
        }
    } else {
        return Err(Error::InvalidData);
    };
    Ok(pressure/100.0)
}

fn compensate_humidity(
    uncompensated: u32,
    calibration: &mut CalibrationData,
) -> Result<f32, Error> {
    let var1: f32 = calibration.t_fine as f32 - 76800.0;
    let var2: f32 =
        calibration.dig_h4 as f32 * 64.0 + (calibration.dig_h5 as f32 / 16384.0) * var1;
    let var3: f32 = uncompensated as f32 - var2;
    let var4: f32 = calibration.dig_h2 as f32 / 65536.0;
    let var5: f32 = 1.0 + (calibration.dig_h3 as f32 / 67108864.0) * var1;
    let var6: f32 = 1.0 + (calibration.dig_h6 as f32 / 67108864.0) * var1 * var5;
    let var6: f32 = var3 * var4 * (var5 * var6);

    let humidity: f32 = var6 * (1.0 - calibration.dig_h1 as f32 * var6 / 524288.0);
    let humidity = if humidity < BME280_HUMIDITY_MIN {
        BME280_HUMIDITY_MIN
    } else if humidity > BME280_HUMIDITY_MAX {
        BME280_HUMIDITY_MAX
    } else {
        humidity
    };
    Ok(humidity)
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
    self.setup()?;
    self.verify_chip_id()?;
    self.calibrate()?;

    Ok(())
  }

  fn verify_chip_id(&mut self) -> Result<(), Error> {
    let chip_id = self.read_reg(BME280_CHIP_ID_ADDR)?;
    if chip_id == BME280_CHIP_ID {
      Ok(())
    } else {
      Err(Error::UnsupportedChip)
    }
  }

  pub fn setup(&mut self) -> Result<(), Error> {
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

    self.write_reg(0xF2, ctrl_hum_reg)?;
    self.write_reg(0xF4, ctrl_meas_reg)?;
    self.write_reg(0xF5, config_reg)?;

    Ok(())
  }

  fn read_reg(&mut self, register: u8) -> Result<u8, Error> {
    let mut data: [u8; 1] = [0];
    let result = self.i2c.write_read(&[register], &mut data);
    match result {
      Ok(_) => Ok(data[0]),
      Err(_) => Err(Error::I2C),
    }
  }

  pub fn write_reg(&mut self, register: u8, data: u8) -> Result<(), Error> {
    let result = self.i2c.smbus_write_byte(register, data);
    match result {
      Ok(_) => Ok(()),
      Err(_) => Err(Error::I2C),
    }
  }

  fn read_data(&mut self, register: u8) -> Result<[u8; BME280_P_T_H_DATA_LEN], Error> {
    let mut data: [u8; BME280_P_T_H_DATA_LEN] = [0; BME280_P_T_H_DATA_LEN];
    let result = self.i2c.write_read(&[register], &mut data);
    match result {
      Ok(_) => Ok(data),
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

  pub fn measure(&mut self) -> Result<Measurements, Error> {
    let measurements = self.read_data(BME280_DATA_ADDR)?;
    match self.calib_data.as_mut() {
      Some(calibration) => {
        let measurements = Measurements::parse(measurements, &mut *calibration)?;
        Ok(measurements)
      }
      None => Err(Error::NoCalibrationData),
    }
  }
}
