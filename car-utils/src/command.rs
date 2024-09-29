use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS, Serialize, Deserialize, Default, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
#[ts(export)]
#[rustfmt::skip]
pub enum Command {
  #[default]
  NOP,
  Statistics, // 获取统计数据
  Navigate { navigate: Navigate, speed: u8, }, // 控制车方向命令，(方向u8, 速度u8)
  TH { enabled: bool, },                       // 是否开启温湿传感器 (enabledu8,)
  Nixie { enabled: bool, brightness: u8, },    // 是否开启 数码管 (enabled u8, brightness)
  Servos { angle: u8, },                       // 舵机 (angle u8, )
  Trace { enabled: bool, },                    // 是否开启寻迹
  Ultrasonic { enabled: bool, },               // 是否开启超声波测距
  Led { enabled: bool, },                      // 是否开启 led
}

impl Command {
  pub fn buf_len(&self) -> usize {
    match self {
      Command::NOP => 0,
      Command::Statistics => 1,
      Command::Navigate { .. } => 3,
      Command::TH { .. } => 2,
      Command::Nixie { .. } => 3,
      Command::Servos { .. } => 2,
      Command::Trace { .. } => 2,
      Command::Ultrasonic { .. } => 2,
      Command::Led { .. } => 2,
    }
  }

  pub fn write(&self, buf: &mut [u8]) {
    debug_assert!(buf.len() == self.buf_len());

    match *self {
      Command::NOP => {
        buf[0] = 0;
      }
      Command::Statistics => {
        debug_assert!(buf.len() == 1);
        buf[0] = 1;
      }
      Command::Navigate { navigate, speed } => {
        debug_assert!(buf.len() == 3);
        buf[0] = 2;
        buf[1] = navigate.to_u8().unwrap_or_default();
        buf[2] = speed;
      }
      Command::TH { enabled } => {
        debug_assert!(buf.len() == 2);
        buf[0] = 3;
        buf[1] = enabled as u8;
      }
      Command::Nixie { enabled, brightness } => {
        debug_assert!(buf.len() == 3);
        buf[0] = 4;
        buf[1] = enabled as u8;
        buf[2] = brightness;
      }
      Command::Servos { angle } => {
        debug_assert!(buf.len() == 2);
        buf[0] = 5;
        buf[1] = angle;
      }
      Command::Trace { enabled } => {
        debug_assert!(buf.len() == 2);
        buf[0] = 6;
        buf[1] = enabled as u8;
      }
      Command::Ultrasonic { enabled } => {
        debug_assert!(buf.len() == 2);
        buf[0] = 7;
        buf[1] = enabled as u8;
      }
      Command::Led { enabled } => {
        debug_assert!(buf.len() == 2);
        buf[0] = 8;
        buf[1] = enabled as u8;
      }
    }
  }

  pub fn parse(buf: &[u8]) -> Result<Command, CommandError> {
    let buf_len = buf.len();
    debug_assert!(!buf.is_empty());
    match buf[0] {
      0 => {
        debug_assert!(buf.len() == 1);
        Ok(Command::NOP)
      }
      1 => {
        debug_assert!(buf.len() == 1);
        Ok(Command::Statistics)
      }
      2 if buf.len() < 3 => Err(CommandError::ParserError),
      2 => {
        debug_assert!(buf_len == 3);
        Ok(Command::Navigate { navigate: Navigate::from_u8(buf[1]).unwrap_or(Navigate::Brake), speed: buf[2] })
      }
      3 if buf_len < 2 => Err(CommandError::ParserError),
      3 => {
        debug_assert!(buf_len == 2);
        Ok(Command::TH { enabled: buf[1] != 0 })
      }
      4 if buf_len < 3 => Err(CommandError::ParserError),
      4 => {
        debug_assert!(buf_len == 3);
        Ok(Command::Nixie { enabled: buf[1] != 0, brightness: buf[2] })
      }
      5 if buf_len < 2 => Err(CommandError::ParserError),
      5 => {
        debug_assert!(buf_len == 2);
        Ok(Command::Servos { angle: buf[1] })
      }
      6 if buf_len < 2 => Err(CommandError::ParserError),
      6 => {
        debug_assert!(buf_len == 2);
        Ok(Command::Trace { enabled: buf[1] != 0 })
      }
      7 if buf_len < 2 => Err(CommandError::ParserError),
      7 => {
        debug_assert!(buf_len == 2);
        Ok(Command::Ultrasonic { enabled: buf[1] != 0 })
      }
      8 if buf_len < 2 => Err(CommandError::ParserError),
      8 => {
        debug_assert!(buf_len == 2);
        Ok(Command::Led { enabled: buf[1] != 0 })
      }
      _ => Err(CommandError::UnknownCommand),
    }
  }
}

#[derive(TS, Serialize, Default, FromPrimitive, ToPrimitive, Debug, Deserialize, Clone, Copy)]
pub enum Angle {
  #[default]
  Normal,
  Left45,
  Left90,
  Right45,
  Right90,
}

// TODO: 重构为宏

#[derive(TS, FromPrimitive, ToPrimitive, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Navigate {
  Brake = 0x00,    // 刹车
  Left = 0x01,     // 左转
  Right = 0x02,    // 右转
  Forward = 0x03,  // 前进
  BackWard = 0x04, // 后退
}

#[derive(Debug)]
pub enum CommandError {
  ParserError,
  UnknownCommand,
}
