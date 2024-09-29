use std::{
  io,
  net::{TcpListener, ToSocketAddrs},
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  thread::{self, JoinHandle},
  time::Duration,
};

use car_utils::Statistics;
use log::{debug, info};

use time::{OffsetDateTime, Time};

use crate::connection::Connection;

#[cfg(feature = "rasp")]
use crate::{
  config,
  driver::{Drivers, Nixie, RgbLed, Ultrasonic, TH},
  trace::follow_line,
};
#[cfg(feature = "rasp")]
use car_utils::command::Navigate;
#[cfg(feature = "rasp")]
use opencv::{
  core::{Mat, MatTraitConst},
  videoio::{VideoCapture, VideoCaptureTrait, VideoCaptureTraitConst, CAP_ANY},
};
#[cfg(feature = "rasp")]
use rppal::gpio::Gpio;

pub struct Context {
  should_shutdown: Arc<AtomicBool>,
  listener: TcpListener,

  statistics: Arc<Statistics>, // 统计信息
  #[cfg(feature = "rasp")]
  drivers: Arc<Drivers>,
}

impl Context {
  pub fn new(addr: impl ToSocketAddrs, #[cfg(feature = "rasp")] drivers: Drivers) -> io::Result<Self> {
    let listener = TcpListener::bind(addr)?;
    info!("listen on: {:?}", listener.local_addr());
    let statistics = Statistics::default();
    statistics.set_servos(90);
    statistics.set_th(true);
    // statistics.set_ultrasonic(true);

    Ok(Self {
      should_shutdown: Arc::new(AtomicBool::new(false)),
      listener,
      statistics: Arc::new(statistics),
      #[cfg(feature = "rasp")]
      drivers: Arc::new(drivers),
    })
  }

  pub fn run(&mut self) -> io::Result<()> {
    let mut threads = vec![
      self.start_statistics_thread(), //
    ];

    // self.statistics.lock().unwrap().trace = true;
    // 接受连接
    while let Ok((stream, addr)) = self.listener.accept() {
      // 每个连接分配一个线程
      info!("incomming: {}", addr);
      stream.set_nodelay(true)?;
      stream.set_nonblocking(true)?;

      #[cfg(feature = "rasp")]
      let drivers = Arc::clone(&self.drivers);

      let statistics = Arc::clone(&self.statistics);
      threads.push(thread::spawn(move || {
        let mut connection = Connection::new(
          stream,
          statistics,
          #[cfg(feature = "rasp")]
          drivers,
        );
        connection.run();
      }));
    }

    // 等待所有线程结束
    threads.into_iter().for_each(|th| {
      let _ = th.join();
    });

    Ok(())
  }
}

#[cfg(not(feature = "rasp"))]
impl Context {
  pub fn start_statistics_thread(&mut self) -> JoinHandle<()> {
    let should_shutdown = Arc::clone(&self.should_shutdown);
    let statistics = Arc::clone(&self.statistics);

    thread::spawn(move || {
      //
      while !should_shutdown.load(Ordering::Acquire) {
        use rand::{thread_rng, Rng};
        let mut thread_rng = thread_rng();

        if statistics.th() {
          statistics.set_temperature(thread_rng.gen_range(20_f32..=30_f32));
          statistics.set_humidity(thread_rng.gen_range(40_f32..=60_f32));
        }

        // 寻迹模块
        // if statistics.trace() {}

        thread::sleep(Duration::from_millis(50)); //
      }
    })
  }
}

#[cfg(feature = "rasp")]
impl Context {
  /// 寻迹线程
  pub fn start_trace_thread(&mut self) -> JoinHandle<()> {
    let mut cap = VideoCapture::new(config::CAMERA_INDEX, CAP_ANY).unwrap();
    if !cap.is_opened().unwrap() {
      panic!("Failed to open camera");
    }

    let should_shutdown = Arc::clone(&self.should_shutdown);
    let statistics = Arc::clone(&self.statistics);
    let driver = Arc::clone(&self.drivers);

    thread::spawn(move || {
      while !should_shutdown.load(Ordering::Acquire) {
        thread::park();

        while statistics.trace() {
          // 寻迹模块
          let mut frame = Mat::default();
          cap.read(&mut frame).unwrap();
          if frame.empty() {
            break;
          }

          let navigate = follow_line(&mut frame);
          debug!("寻迹：{:?}", navigate);
          // let montor = driver.montor.lock().unwrap();
          driver.montor.lock().unwrap().navigate(navigate, statistics.speed());
          // imgcodecs::imwrite("frame.png", &frame, &Vector::new()).unwrap();
        }

        driver.montor.lock().unwrap().navigate(Navigate::Brake, statistics.speed());
      }
    })
  }

  pub fn start_statistics_thread(&mut self) -> JoinHandle<()> {
    let driver = Arc::clone(&self.drivers);
    let should_shutdown = Arc::clone(&self.should_shutdown);
    let statistics = Arc::clone(&self.statistics);

    let led_thread = self.start_led_thread();
    let nixie_thread = self.start_nixie_thread();
    let trace_thread = self.start_trace_thread();

    let gpio = Gpio::new().unwrap();
    let mut th = TH::new(&gpio);
    let mut ultrasonic = Ultrasonic::new(&gpio);
    thread::spawn(move || {
      //
      while !should_shutdown.load(Ordering::Acquire) {
        // 温湿传感器
        if statistics.th() {
          // debug!("温室传感器: {:?}", th.measure());
          if let Some(m) = th.measure() {
            // debug!("温室传感器: {:?}", m);
            statistics.set_temperature(m.temperature.floor());
            statistics.set_humidity(m.humidity.floor());
          }
        }

        // 超声波模块
        if statistics.ultrasonic() {
          let distance = ultrasonic.get_distance(statistics.th().then_some(statistics.temperature()));
          if let Some(distance) = distance {
            statistics.set_distance(distance);
          }
          let mut montor = driver.montor.lock().unwrap();
          if statistics.servos() == 90
            && statistics.distance() <= config::MIN_DISTANCE
            && montor.navigate == Navigate::Forward
          {
            println!("小于该距离");
            montor.navigate(Navigate::Brake, statistics.speed());
          }
        }
        // 寻迹{}
        if statistics.trace() {
          trace_thread.thread().unpark();
        }

        // 数码管状态
        if statistics.led() {
          led_thread.thread().unpark();
        }

        if statistics.nixie() {
          nixie_thread.thread().unpark();
        }

        thread::sleep(Duration::from_millis(50)); //
      }

      // 等待所有线程结束
      trace_thread.join().unwrap();
      led_thread.join().unwrap();
      nixie_thread.join().unwrap();
    })
  }

  /// FIX: 不够实时
  /// 开启
  pub fn start_led_thread(&mut self) -> JoinHandle<()> {
    let should_shutdown = Arc::clone(&self.should_shutdown);
    let statistics = Arc::clone(&self.statistics);

    let gpio = Gpio::new().unwrap();
    let mut rgb_led = RgbLed::new(&gpio);

    thread::spawn(move || {
      while !should_shutdown.load(Ordering::Acquire) {
        thread::park();
        while statistics.led() {
          rgb_led.red.set_high();
          thread::sleep(Duration::from_secs(1));
          rgb_led.green.set_high(); // RG
          thread::sleep(Duration::from_secs(1));
          rgb_led.green.set_low();
          rgb_led.blue.set_high(); // RB
          thread::sleep(Duration::from_secs(1));
          rgb_led.red.set_low();
          rgb_led.blue.set_low();
          rgb_led.green.set_high(); // G
          thread::sleep(Duration::from_secs(1));
          rgb_led.blue.set_high(); // GB
          thread::sleep(Duration::from_secs(1));
          rgb_led.green.set_low(); // B
          thread::sleep(Duration::from_secs(1));
          rgb_led.red.set_high();
          rgb_led.blue.set_high();
          thread::sleep(Duration::from_secs(1));
        }

        rgb_led.off();
      }
    })
  }

  pub fn start_nixie_thread(&mut self) -> JoinHandle<()> {
    let should_shutdown = Arc::clone(&self.should_shutdown);
    let statistics = Arc::clone(&self.statistics);

    let gpio = Gpio::new().unwrap();
    let mut nixie = Nixie::new(&gpio);
    thread::spawn(move || {
      //
      while !should_shutdown.load(Ordering::Acquire) {
        thread::park();
        while statistics.nixie() {
          let now = get_local_time();
          nixie.display_time(&now, true, statistics.nixie_brightness());
          thread::sleep(Duration::from_millis(500));
          nixie.display_time(&now, false, statistics.nixie_brightness());
          thread::sleep(Duration::from_millis(500));
        }
      }

      nixie.off();
    })
  }
}

/// FIX: OffsetDateTime::now_local()
fn get_local_time() -> Time {
  (OffsetDateTime::now_utc() + Duration::from_secs(8 * 60 * 60)).time()
}
