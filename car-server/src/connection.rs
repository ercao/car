use std::{
  io::{self, ErrorKind, Read, Write},
  net::TcpStream,
  sync::Arc,
  thread::sleep,
  time::{Duration, Instant},
};

use car_utils::{
  buffer::RingBuffer,
  command::{Command, Navigate},
  ResponseType, Statistics, REQUEST_HEADER_LEN, RESPONSE_HEADER_LEN,
};
use log::{debug, info};
use serde::Serialize;

use crate::config;
#[cfg(feature = "rasp")]
use crate::driver::Drivers;

pub struct Connection {
  stream: TcpStream,

  tx_buffer: RingBuffer<1024>,
  rx_buffer: RingBuffer<1024>,

  statistics: Arc<Statistics>,
  instant: Instant, // 下一次推送统计数据的时间

  #[cfg(feature = "rasp")]
  drivers: Arc<Drivers>,
}

impl Connection {
  pub fn new(
    stream: TcpStream,
    statistics: Arc<Statistics>,
    #[cfg(feature = "rasp")] drivers: Arc<Drivers>,
  ) -> Connection {
    Connection {
      stream,
      tx_buffer: RingBuffer::new(),
      rx_buffer: RingBuffer::new(),
      statistics,
      instant: Instant::now(),
      #[cfg(feature = "rasp")]
      drivers,
    }
  }

  /// 运行
  pub fn run(&mut self) {
    loop {
      // 处理请求
      let should_shutdown = self
        .rx_buffer
        .enqueue_with(|buf| {
          self.stream.read(buf).map_or_else(
            |err| {
              if matches!(err.kind(), ErrorKind::WouldBlock) {
                (0, false)
              } else {
                info!("{:?}", err.kind());
                (0, true)
              }
            },
            |len| {
              if len == 0 {
                debug!("关闭连接");
                (len, true)
              } else {
                (len, false)
              }
            },
          )
        })
        .unwrap(); // TODO: FULL

      if should_shutdown {
        break;
      }

      // 负载的最大长度 256
      while self.rx_buffer.peek_one().map_or(false, |len| self.rx_buffer.len() >= len as usize + REQUEST_HEADER_LEN) {
        let len = self.rx_buffer.dequeue_one().unwrap() as usize;
        self.rx_buffer.congestion_alloced(len);
        self.rx_buffer.dequeue_with(len, |_payload| {
          let command = Command::parse(_payload)
            .inspect_err(|err| {
              println!("{:?}", err);
            })
            .unwrap_or_default();

          request_handler(
            #[cfg(feature = "rasp")]
            &self.drivers,
            &self.statistics,
            command,
          );
        });

        self.send_statistics(); // 每一次请求都回复一次统计数据
      }

      // 检查是需要发送统计数据
      let now = Instant::now();
      if self.instant <= now {
        self.send_statistics();
      }

      // 发送数据
      let _ = self.send();
      sleep(Duration::from_millis(10));
    }
  }

  /// 将 tx_buffer 中的包发送出去
  fn send(&mut self) -> io::Result<()> {
    let max_len = self.tx_buffer.len();
    self.tx_buffer.dequeue_with(max_len, |buf| {
      //
      self.stream.write_all(buf)
    })
  }

  /// 发送统计信息
  fn send_statistics(&mut self) {
    let now = Instant::now();
    self.send_response(ResponseType::Statistics, &self.statistics.to_response());
    self.instant = now + Duration::from_secs(1);
  }

  /// 发送回复
  fn send_response<T: Serialize>(&mut self, response_type: ResponseType, payload: &T) {
    let payload = serde_json::json!(payload).to_string();
    let payload = payload.as_bytes();
    if payload.len() > u8::MAX as usize {
      return;
    }

    if self.tx_buffer.free_len() >= payload.len() + RESPONSE_HEADER_LEN {
      self.tx_buffer.enqueue_one(payload.len() as u8).unwrap();
      self.tx_buffer.enqueue_one(response_type as u8).unwrap();

      let payload = self
        .tx_buffer
        .enqueue_with(|buf| {
          let max_len = usize::min(buf.len(), payload.len());
          buf[..max_len].copy_from_slice(&payload[..max_len]);

          (max_len, &payload[max_len..])
        })
        .unwrap();

      self
        .tx_buffer
        .enqueue_with(|buf| {
          buf[..payload.len()].copy_from_slice(payload);
          (payload.len(), ())
        })
        .unwrap();
    }

    if self.tx_buffer.is_empty() {

      // self.stream.write_all(&[&[payload.len() as u8, command as u8], payload].concat())
    }
  }
}

/// 请求处理函数
fn request_handler(
  #[cfg(feature = "rasp")] drivers: &Arc<Drivers>, //
  statistics: &Arc<Statistics>,
  command: Command,
) {
  debug!("command: {:?}", command);

  match command {
    Command::NOP => {}
    Command::Statistics => {}
    Command::Navigate { mut navigate, speed } => {
      if statistics.ultrasonic()
        && statistics.distance() <= config::MIN_DISTANCE
        && navigate == Navigate::Forward
        && statistics.servos() == 90
      {
        debug!("障碍物");
        navigate = Navigate::Brake; // 小于该距离就刹车
      }
      statistics.set_speed(speed);

      #[cfg(feature = "rasp")]
      drivers.montor.lock().unwrap().navigate(navigate, speed);
    }
    Command::TH { enabled } => {
      statistics.set_th(enabled);
    }
    Command::Nixie { enabled, brightness } => {
      statistics.set_nixie(enabled);
      if enabled {
        statistics.set_nixie_brightness(brightness);
      }
    }
    Command::Servos { angle } => {
      #[cfg(feature = "rasp")]
      if drivers.servos.lock().unwrap().rotate(angle) {
        statistics.set_servos(angle)
      };

      #[cfg(not(feature = "rasp"))]
      statistics.set_servos(angle);
    }
    Command::Trace { enabled } => {
      statistics.set_trace(enabled);
    }
    Command::Ultrasonic { enabled } => {
      statistics.set_ultrasonic(enabled);
    }
    Command::Led { enabled } => {
      statistics.set_led(enabled);
    }
  }
}
