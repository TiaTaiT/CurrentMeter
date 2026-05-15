// File: /app_firmware/src/hardware/init.rs
pub mod init;
pub mod leds;

pub use init::{init, Hardware};
pub use leds::StatusLeds;
