// File: /app_firmware/src/main.rs
#![no_std]
#![no_main]

mod hardware;

use app_core::{
    constants::MODBUS_SLAVE_ADDR,
    adc_converter::StoredValues,
    hardware_traits::{ModbusRxInterface, ModbusTxInterface, PowerState},
    modbus::{handle_request, RequestError},
};
use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use defmt_rtt as _;
use panic_halt as _;

use crate::hardware::{Hardware, StatusLeds, SystemSensor};

static STORE: StoredValues = StoredValues::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // 1. Initialize new Hardware struct
    let hw: Hardware = hardware::init();
    let leds = hw.leds;
    let sensors = hw.sensors;
    let modem_tx = hw.modem_tx;
    let modem_rx = hw.modem_rx;
    let modem_ctrl = hw.modem_ctrl;

    spawner.spawn(task1(leds).unwrap());
    spawner.spawn(task2(sensors).unwrap());
    spawner.spawn(modbus_task(modem_rx, modem_tx, modem_ctrl).unwrap());
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

        STORE.update(voltages, currents);

        info!("-----------------------------");
        info!("Currents: {:?}", STORE.get_currents());
        info!("Voltages: {:?}", STORE.get_voltages());

        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn modbus_task(
    mut modem_rx: hardware::ModemRx,
    mut modem_tx: hardware::ModemTx,
    mut modem_ctrl: hardware::ModemControl,
) {
    let slave_address = MODBUS_SLAVE_ADDR as u8;
    let mut rx_buf = [0u8; 256];
    let mut tx_buf = [0u8; 256];

    modem_ctrl.set_receive_mode();

    loop {
        let bytes_read = match modem_rx.read_until_idle(&mut rx_buf).await {
            Ok(size) if size > 0 => size,
            Ok(_) => continue,
            Err(_) => {
                warn!("Modbus RX failed");
                continue;
            }
        };

        let snapshot = STORE.snapshot();
        match handle_request(&rx_buf[..bytes_read], slave_address, &snapshot, &mut tx_buf) {
            Ok(Some(response_len)) => {
                modem_ctrl.set_transmit_mode();
                if modem_tx.write(&tx_buf[..response_len]).await.is_err() {
                    warn!("Modbus TX failed");
                }
                modem_ctrl.set_receive_mode();
            }
            Ok(None) => {}
            Err(RequestError::FrameTooShort) | Err(RequestError::CrcMismatch) => {}
            Err(_) => warn!("Modbus request handling failed"),
        }
    }
}
