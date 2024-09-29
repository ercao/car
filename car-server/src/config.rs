use std::net::Ipv4Addr;

pub const LISTEN_ADDR: Ipv4Addr = Ipv4Addr::UNSPECIFIED; // 监听地址
pub const LISTEN_PORT: u16 = 5000; // 监听端口号
                                   //

pub const MIN_DISTANCE: u16 = 20; // 可以距障碍物的最小距离 cm
                                  // 引脚配置 BCM 编号
                                  // RGB LED 灯

pub const CAMERA_INDEX: i32 = 0; // 寻迹摄像头

pub const PIN_LED_RED: u8 = 22;
pub const PIN_LED_GREEN: u8 = 27;
pub const PIN_LED_BLUE: u8 = 17;

// L298n 驱动板
pub const PIN_L298N_IN1: u8 = 5;
pub const PIN_L298N_IN2: u8 = 6;
pub const PIN_L298N_IN3: u8 = 13;
pub const PIN_L298N_IN4: u8 = 19;

// TM1637: 4位数码管
pub const PIN_TM1637_CLK: u8 = 16;
pub const PIN_TM1637_DIO: u8 = 20;

// DHT11：温湿度传感器
pub const PIN_DHT11_DATA: u8 = 4;

// HC-SRC04: 超声波
pub const PIN_HCSRC04_TRIG: u8 = 23;
pub const PIN_HCSRC04_ECHO: u8 = 24;

// 蜂鸣器
// pub const PIN_BUZZER_CTL: u8 = 0;

// 舵机控制
pub const PIN_SERVOS_CTL: u8 = 26;
