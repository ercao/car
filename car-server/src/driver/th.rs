//! DHT11 传感器: 传感温度与湿度

use embedded_hal::delay::DelayNs;
use rppal::{
  gpio::{Gpio, IoPin, Mode},
  hal::Delay,
};

use crate::config;

#[derive(Debug)]
pub enum Error {
  NoResponse,
  ChecksumMismatch,
  Timeout,
}

#[derive(Debug)]
pub struct Measurement {
  pub temperature: f32,
  pub humidity: f32,
}

/// 温湿度传感器
pub struct TH {
  pin: IoPin, // data pin
  delay: Delay,
}

impl TH {
  pub fn new(gpio: &Gpio) -> Self {
    Self { pin: gpio.get(config::PIN_DHT11_DATA).unwrap().into_io(rppal::gpio::Mode::Input), delay: Delay::new() }
  }

  /// 测量温度与湿度
  pub fn measure(&mut self) -> Option<Measurement> {
    self
      .read_raw()
      .map(|[hi, hf, ti, tf]| Measurement {
        temperature: format!("{:}.{:}", ti, tf).parse::<f32>().unwrap(),
        humidity: format!("{:}.{:}", hi, hf).parse::<f32>().unwrap(),
      })
      .inspect_err(|_err| {
        // println!("th: {:?}", err);
      })
      .ok()
  }

  fn read_bit(&mut self) -> Result<u8, Error> {
    wait_until_timeout(&mut self.delay, || self.pin.is_high(), 100)?;
    self.delay.delay_us(28);
    let bit = self.pin.is_high() as u8;
    wait_until_timeout(&mut self.delay, || self.pin.is_low(), 100)?;
    Ok(bit)
  }

  fn read_byte(&mut self) -> Result<u8, Error> {
    let mut byte: u8 = 0;
    for _ in 0..8 {
      byte <<= 1;
      byte |= self.read_bit()?;
    }
    Ok(byte)
  }

  fn read_raw(&mut self) -> Result<[u8; 4], Error> {
    self.pin.set_mode(Mode::Output);
    self.delay.delay_us(140);
    self.pin.set_high();
    self.delay.delay_us(140);

    self.pin.set_low();
    self.delay.delay_ms(18_u32);
    self.pin.set_high();
    self.delay.delay_us(48_u32);

    self.pin.set_mode(Mode::Input);
    if self.pin.is_low() {
      wait_until_timeout(&mut self.delay, || self.pin.is_high(), 100)?;
      wait_until_timeout(&mut self.delay, || self.pin.is_low(), 100)?;

      let mut data = [0; 4];
      for b in data.iter_mut() {
        *b = self.read_byte()?;
      }

      let checksum = self.read_byte()?;
      if data.iter().fold(0u8, |sum, v| sum.wrapping_add(*v)) != checksum {
        Err(Error::ChecksumMismatch)
      } else {
        Ok(data)
      }
    } else {
      Err(Error::NoResponse)
    }
  }
}

/// wait until the given function returns true or the timeout is reached.
fn wait_until_timeout(delay: &mut Delay, func: impl Fn() -> bool, timeout_us: u8) -> Result<(), Error> {
  for _ in 0..timeout_us {
    if func() {
      return Ok(());
    }

    delay.delay_us(1_u32);
  }
  Err(Error::Timeout)
}
