use hc_sr04::HcSr04;
use log::debug;
use rppal::gpio::Gpio;

use crate::config;

pub struct Ultrasonic {}

impl Ultrasonic {
  pub fn new(_gpio: &Gpio) -> Self {
    Self {}
  }

  /// FIX: 只有第一次测是相对准确的
  /// 获取距离 cm
  pub fn get_distance(&mut self, _temperature: Option<f32>) -> Option<u16> {
    let mut hcsrc04 = HcSr04::new(
      config::PIN_HCSRC04_TRIG,
      config::PIN_HCSRC04_ECHO,
      _temperature, //
    )
    .expect("失败");

    hcsrc04
      .measure_distance(hc_sr04::Unit::Centimeters)
      .inspect_err(|e| {
        debug!("get_distance: {}", e); //
      })
      .unwrap_or_default()
      .map(|x| x.floor() as u16)
  }
}
