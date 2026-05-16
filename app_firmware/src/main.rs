// File: /app_firmware/src/main.rs
#![no_std]
#![no_main]

mod hardware;

use defmt::info;
use app_core::hardware_traits::PowerState;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use app_core::adc_converter::StoredValues;

use defmt_rtt as _;
use panic_halt as _;

use crate::hardware::{Hardware, StatusLeds, SystemSensor};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // 1. Initialize new Hardware struct
    let hw: Hardware = hardware::init();
    let leds = hw.leds;
    let sensors = hw.sensors;

    let store = StoredValues::new();

    spawner.spawn(task1(leds).unwrap());
    spawner.spawn(task2(sensors, store).unwrap());
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
async fn task2(mut sensors: SystemSensor, store: StoredValues) {
    loop {
        let currents = sensors.read_currents().await;
        let voltages = sensors.read_voltages().await;

        store.update(voltages, currents);

        info!("-----------------------------");
        info!("Currents: {:?}", store.get_currents());
        info!("Voltages: {:?}", store.get_voltages());

        Timer::after(Duration::from_secs(1)).await;
    }
}
