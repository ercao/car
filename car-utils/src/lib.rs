pub mod buffer;
pub mod command;

use std::sync::atomic::{AtomicBool, AtomicI16, AtomicU16, AtomicU8, Ordering};

use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[repr(u8)]
#[derive(TS, FromPrimitive, ToPrimitive, Debug, Deserialize, Clone, Copy)]
#[ts(export)]
pub enum CommandType {
  NOP,
  Statistics, // 获取统计数据
  Navigate,   // 控制车方向命令，(方向u8, 速度u8)
  TH,         // 是否开启温湿传感器 (enabledu8,)
  Nixie,      // 是否开启 数码管 (enabled u8, brightness)
  Servos,     // 舵机 (angle u8, )
  Trace,      // 是否开启寻迹
  Ultrasonic, // 是否开启超声波测距
}

#[derive(FromPrimitive, ToPrimitive, Debug)]
pub enum ResponseType {
  Statistics,
}

/// 统计信息
#[derive(TS, Serialize, Deserialize, Clone, Copy, Default, Debug)]
#[ts(export)]
pub struct Response {
  // 数码管
  // pub now: (u8, u8, u8),           // 时间 （hms）
  pub time_brightness: Option<u8>, // 时间显示亮度

  // 蜂鸣器

  // 电动机
  pub speed_percent: u8, // 速度百分比

  // 超声波测距 舵机
  pub distance: Option<f32>, // 障碍物距离
  pub servos: u8,            // 舵机角度

  // LED 灯
  pub led: bool, // 是否开启LED 等

  // 温湿传感器
  pub th: Option<(f32, f32)>, // 温度, 湿度
  // 寻迹
  pub trace: bool, // 是否开启寻迹
}

/// 统计信息
#[derive(Default, Debug)]
pub struct Statistics {
  // 数码管
  nixie: AtomicBool,
  nixie_brightness: AtomicU8,

  // 电动机
  speed: AtomicU8,

  // 超声波测距 舵机
  ultrasonic: AtomicBool,
  distance: AtomicU16,
  servos: AtomicU8,

  // LED 灯
  led: AtomicBool,

  // 温湿传感器
  th: AtomicBool,
  temperature: AtomicI16,
  humidity: AtomicU16,

  // 寻迹
  trace: AtomicBool,
}

macro_rules! getter_setter {
  ($field:ident, $setter:ident, $type:ty) => {
    pub fn $field(&self) -> $type {
      self.$field.load(Ordering::SeqCst)
    }

    pub fn $setter(&self, value: $type) {
      self.$field.store(value, Ordering::SeqCst);
    }
  };
}

impl Statistics {
  getter_setter!(nixie, set_nixie, bool);
  getter_setter!(nixie_brightness, set_nixie_brightness, u8);
  getter_setter!(speed, set_speed, u8);
  getter_setter!(ultrasonic, set_ultrasonic, bool);
  getter_setter!(distance, set_distance, u16);
  getter_setter!(servos, set_servos, u8);
  getter_setter!(led, set_led, bool);
  getter_setter!(th, set_th, bool);
  getter_setter!(trace, set_trace, bool);

  pub fn temperature(&self) -> f32 {
    self.temperature.load(Ordering::SeqCst) as f32 / 100_f32
  }
  pub fn set_temperature(&self, value: f32) {
    self.temperature.store((value * 100_f32) as i16, Ordering::SeqCst)
  }
  pub fn humidity(&self) -> f32 {
    self.humidity.load(Ordering::SeqCst) as f32 / 100_f32
  }
  pub fn set_humidity(&self, value: f32) {
    self.humidity.store((value * 100_f32) as u16, Ordering::SeqCst)
  }

  pub fn to_response(&self) -> Response {
    Response {
      time_brightness: self.nixie().then(|| self.nixie_brightness()),
      speed_percent: self.speed(),
      distance: self.ultrasonic().then(|| self.distance() as f32 / 100_f32),
      servos: self.servos(),
      led: self.led(),
      th: self.th().then(|| (self.temperature(), self.humidity())),
      trace: self.trace(),
    }
  }
}

pub const RESPONSE_HEADER_LEN: usize = 2; // 回复包头长度
pub const REQUEST_HEADER_LEN: usize = 1; // 请求包头长度
