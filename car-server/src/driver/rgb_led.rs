//! RGB LED

use rppal::gpio::{Gpio, OutputPin};

use crate::config;

/// TODO: 设置亮度
pub struct RgbLed {
  pub red: OutputPin,
  pub green: OutputPin,
  pub blue: OutputPin,
}

impl RgbLed {
  //
  pub fn new(gpio: &Gpio) -> Self {
    Self {
      red: gpio.get(config::PIN_LED_RED).unwrap().into_output_low(),
      green: gpio.get(config::PIN_LED_GREEN).unwrap().into_output_low(),
      blue: gpio.get(config::PIN_LED_BLUE).unwrap().into_output_low(),
    }
  }

  /// 关闭灯
  pub fn off(&mut self) {
    self.red.set_low();
    self.green.set_low();
    self.blue.set_low();
  }
}
