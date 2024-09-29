use log::debug;
use rppal::{
  gpio::{Gpio, IoPin, Mode, OutputPin},
  hal::Delay,
};
use time::Time;
use tm1637_embedded_hal::{
  blocking::TM1637,
  mappings::{DigitBits, SegmentBits},
  Brightness,
};

use crate::config;

/// TM1637: 4位LED 数码管
pub struct Nixie {
  tm1637: TM1637<OutputPin, IoPin, Delay>,
}

impl Nixie {
  pub fn new(gpio: &Gpio) -> Self {
    let clk = gpio.get(config::PIN_TM1637_CLK).unwrap().into_output_low();
    let dio = gpio.get(config::PIN_TM1637_DIO).unwrap().into_io(Mode::Output);

    Self { tm1637: TM1637::builder(clk, dio, Delay).brightness(Brightness::L0).build() }
  }

  /// 显示时间
  /// flag: 是否显示冒号
  pub fn display_time(&mut self, time: &Time, has_colon: bool, brightness: u8) {
    let brightness = match brightness {
      0 => Brightness::L0,
      1 => Brightness::L1,
      2 => Brightness::L2,
      3 => Brightness::L3,
      4 => Brightness::L4,
      5 => Brightness::L5,
      6 => Brightness::L6,
      7 => Brightness::L7,
      brightness => {
        debug!("非有效亮度等级: {:}", brightness);
        return;
      }
    };

    let (h, m, _) = time.as_hms();
    let bytes = [
      DigitBits::from_digit(h / 10) as u8,
      DigitBits::from_digit(h % 10) as u8 | (has_colon as u8 * (SegmentBits::SegPoint as u8)),
      DigitBits::from_digit(m / 10) as u8,
      DigitBits::from_digit(m % 10) as u8,
    ];

    let _ = self.tm1637.write_brightness(brightness).inspect_err(|_| {
      debug!("nixie brightness setting failed");
    });
    let _ = self.tm1637.write_segments_raw(0, &bytes);
  }

  /// 关闭显示
  pub fn off(&mut self) {
    let _ = self.tm1637.off();
  }
}
