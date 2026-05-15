// File: /app_firmware/src/main.rs
#![no_std]
#![no_main]

mod hardware;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use defmt::*;
use defmt_rtt as _;
use panic_halt as _;

use crate::hardware::init::Hardware;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // 1. Initialize new Hardware struct
    let mut hw: Hardware = hardware::init::init();

    spawner.spawn(task1().unwrap());
}

#[embassy_executor::task]
async fn task1() {
    loop {
        info!("Hello from task1!");
        Timer::after(Duration::from_secs(1)).await;
    }
}
