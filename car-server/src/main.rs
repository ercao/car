mod config;
mod connection;
mod context;
#[cfg(feature = "rasp")]
mod driver;
#[cfg(feature = "rasp")]
mod trace;

use context::Context;
use log::LevelFilter;
#[cfg(feature = "rasp")]
use rppal::gpio::Gpio;
use std::io;

/// TODO: 优雅的结束进程
fn main() -> io::Result<()> {
  env_logger::builder().filter_level(LevelFilter::Debug).init();

  #[cfg(feature = "rasp")]
  let gpio = Gpio::new().unwrap();
  #[cfg(feature = "rasp")]
  let driver = driver::Drivers::new(&gpio);

  // #[cfg(feature = "rasp")]
  // loop {
  //   debug!("获取的距离: {:?}", driver.ultrasonic.lock().unwrap().get_distance(None));

  //   thread::sleep(std::time::Duration::from_millis(50));
  // }

  let mut context = Context::new(
    (config::LISTEN_ADDR, config::LISTEN_PORT),
    #[cfg(feature = "rasp")]
    driver,
  )
  .unwrap();

  context.run().unwrap();
  unreachable!("listener accept() block ?");
}
