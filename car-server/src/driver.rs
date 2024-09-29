// mod buzzer;
mod montor;
mod nixie;
mod rgb_led;
mod servos;
mod th;
// mod trace;
mod ultrasonic;

pub use montor::Montor;
pub use nixie::Nixie;
pub use rgb_led::RgbLed;
pub use servos::Servos;
pub use std::sync::Mutex;
pub use th::TH;
pub use ultrasonic::Ultrasonic;

use rppal::gpio::Gpio;

/// 整合所有驱动驱动
pub struct Drivers {
  // pub buzzer: Mutex<Buzzer>,
  pub montor: Mutex<Montor>,
  // pub nixie: Mutex<Nixie>,
  pub servos: Mutex<Servos>,
  // pub th: Mutex<TH>,
  // pub ultrasonic: Mutex<Ultrasonic>,
}

impl Drivers {
  pub fn new(gpio: &Gpio) -> Self {
    Self {
      // buzzer: Mutex::new(Buzzer::new(gpio)),
      montor: Mutex::new(Montor::new(gpio)),
      // nixie: Mutex::new(Nixie::new(gpio)),
      servos: Mutex::new(Servos::new(gpio)),
      // th: Mutex::new(TH::new(gpio)),
      // ultrasonic: Mutex::new(Ultrasonic::new(gpio)),
    }
  }
}
