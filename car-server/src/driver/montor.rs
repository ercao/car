//! L298n 驱动板: 驱动电机转动速度

use car_utils::command::Navigate;
use rppal::gpio::{Gpio, OutputPin};

use crate::config;

/// INT2, INT4 == 1; 前进
pub struct Montor {
  in1: OutputPin,
  in2: OutputPin,
  in3: OutputPin,
  in4: OutputPin,

  pub navigate: Navigate, // 当前状态
  frequency: f64,         // 频率
}

impl Montor {
  pub const DEFAULT_FREQUENCY: f64 = 40_f64; // 默认 40HZ

  // pub const DEFAULT_DUTY_CYCLE: f64 = 0.2_f64;

  pub fn new(gpio: &Gpio) -> Self {
    Self {
      in1: gpio.get(config::PIN_L298N_IN1).unwrap().into_output_low(),
      in2: gpio.get(config::PIN_L298N_IN2).unwrap().into_output_low(),
      in3: gpio.get(config::PIN_L298N_IN3).unwrap().into_output_low(),
      in4: gpio.get(config::PIN_L298N_IN4).unwrap().into_output_low(),
      frequency: Self::DEFAULT_FREQUENCY,
      navigate: Navigate::Brake,
    }
  }

  /// speed 0-100
  pub fn navigate(&mut self, navigate: Navigate, mut speed: u8) {
    // if matches!(navigate, Navigate::Left | Navigate::Right) {
    //   speed = (speed << 1).min(100); // 左转右转速度加倍
    // }
    let duty_cycle = speed as f64 / 100.0;

    match navigate {
      Navigate::Brake => {
        self.set_left_stop();
        self.set_right_stop();
      }
      Navigate::Left => {
        self.set_left_backward(duty_cycle);
        self.set_right_forward(duty_cycle);
      }
      Navigate::Right => {
        self.set_left_forward(duty_cycle);
        self.set_right_backward(duty_cycle);
      }
      Navigate::Forward => {
        self.set_left_forward(duty_cycle);
        self.set_right_forward(duty_cycle);
      }
      Navigate::BackWard => {
        self.set_left_backward(duty_cycle);
        self.set_right_backward(duty_cycle);
      }
    }

    self.navigate = navigate;
  }

  pub fn set_left_forward(&mut self, duty_cycle: f64) {
    self.in1.clear_pwm().unwrap();
    self.in2.set_pwm_frequency(self.frequency, duty_cycle).unwrap();
  }

  fn set_left_stop(&mut self) {
    self.in1.clear_pwm().unwrap();
    self.in2.clear_pwm().unwrap();
  }

  fn set_left_backward(&mut self, duty_cycle: f64) {
    self.in1.set_pwm_frequency(self.frequency, duty_cycle).unwrap();
    self.in2.clear_pwm().unwrap();
  }

  fn set_right_forward(&mut self, mut duty_cycle: f64) {
    duty_cycle *= 2.0;

    self.in3.clear_pwm().unwrap();
    self.in4.set_pwm_frequency(self.frequency, duty_cycle).unwrap();
  }

  fn set_right_stop(&mut self) {
    self.in3.clear_pwm().unwrap();
    self.in4.clear_pwm().unwrap();
  }

  fn set_right_backward(&mut self, mut duty_cycle: f64) {
    duty_cycle *= 2.0;

    self.in3.set_pwm_frequency(self.frequency, duty_cycle).unwrap();
    self.in4.clear_pwm().unwrap();
  }
}
