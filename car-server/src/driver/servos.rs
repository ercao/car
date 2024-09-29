//! 舵机控制

use std::time::Duration;

use log::debug;
use rppal::gpio::{Gpio, OutputPin};

use crate::config;

/// 舵机控制
pub struct Servos {
  ctl: OutputPin,
}

impl Servos {
  pub const PULSE_WIDTH_ANGLE_0: Duration = Duration::from_micros(500); // -90
  pub const PULSE_WIDTH_ANGLE_45: Duration = Duration::from_micros(1000);
  pub const PULSE_WIDTH_ANGLE_90: Duration = Duration::from_micros(1500);
  pub const PULSE_WIDTH_ANGLE_135: Duration = Duration::from_micros(2000);
  pub const PULSE_WIDTH_ANGLE_180: Duration = Duration::from_micros(2500);
  pub const PERIOD: Duration = Duration::from_millis(20); // 20ms
  pub const DEFAULT_ANGLE: u8 = 90;

  pub fn new(gpio: &Gpio) -> Self {
    let ctl = gpio.get(config::PIN_SERVOS_CTL).unwrap().into_output_low();
    let mut servos = Self { ctl };
    servos.rotate(Self::DEFAULT_ANGLE); // 恢复到正常位置
    servos
  }

  /// 转动
  pub fn rotate(&mut self, angle: u8) -> bool {
    let pulse_width = match angle {
      0 => Self::PULSE_WIDTH_ANGLE_0,
      45 => Self::PULSE_WIDTH_ANGLE_45,
      90 => Self::PULSE_WIDTH_ANGLE_90,
      135 => Self::PULSE_WIDTH_ANGLE_135,
      180 => Self::PULSE_WIDTH_ANGLE_180,
      _ => {
        debug!("servos: 不可用的角度");
        return false;
      }
    };

    self.ctl.set_pwm(Self::PERIOD, pulse_width).inspect_err(|e| debug!("{}", e)).is_ok()
  }
}
