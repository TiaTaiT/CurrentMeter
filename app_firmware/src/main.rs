// File: /app_firmware/src/main.rs
#![no_std]
#![no_main]

mod hardware;

use defmt::info;
use app_core::hardware_traits::PowerState;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use defmt_rtt as _;
use panic_halt as _;

use crate::hardware::{Hardware, StatusLeds, SystemSensor, sensors};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // 1. Initialize new Hardware struct
    let hw: Hardware = hardware::init();
    let leds = hw.leds;
    let sensors = hw.sensors;

    spawner.spawn(task1(leds).unwrap());
    spawner.spawn(task2(sensors).unwrap());
}

#[embassy_executor::task]
async fn task1(mut leds: StatusLeds) {
    loop {
        leds.set_sys_led(PowerState::On);
        Timer::after(Duration::from_millis(200)).await;
        leds.set_sys_led(PowerState::Off);
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn task2(mut sensors: SystemSensor) {
    loop {
        let currents = sensors.read_currents().await;
        let voltages = sensors.read_voltages().await;

        info!("-----------------------------");
        info!("Currents: {:?}", currents);
        info!("Voltages: {:?}", voltages);

        Timer::after(Duration::from_secs(1)).await;
    }
}
