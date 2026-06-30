// File: /app_firmware/src/main.rs
#![no_std]
#![no_main]

mod hardware;

use app_core::{
    constants::MODBUS_SLAVE_ADDR,
    adc_converter::StoredValues,
    hardware_traits::{ModbusTxInterface, PowerState},
    modbus::{crc16, handle_request, RequestError, READ_INPUT_REGISTERS},
};
use defmt::{debug, info, warn};
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
        let bytes_read = match modem_rx.read_until_idle_detailed(&mut rx_buf).await {
            Ok(size) if size > 0 => size,
            Ok(_) => continue,
            Err(error) => {
                warn!("Modbus RX failed on UART1: error={:?}", error);
                continue;
            }
        };

        debug!(
            "Modbus RX frame on UART1: len={}, data={=[u8]:x}",
            bytes_read,
            &rx_buf[..bytes_read]
        );

        let snapshot = STORE.snapshot();
        match handle_request(&rx_buf[..bytes_read], slave_address, &snapshot, &mut tx_buf) {
            Ok(Some(response_len)) => {
                debug!(
                    "Modbus TX frame on UART1: len={}, data={=[u8]:x}",
                    response_len,
                    &tx_buf[..response_len]
                );
                modem_ctrl.set_transmit_mode();
                if modem_tx.write(&tx_buf[..response_len]).await.is_err() {
                    warn!("Modbus TX failed");
                }
                modem_ctrl.set_receive_mode();
            }
            Ok(None) => {}
            Err(RequestError::FrameTooShort) => {
                warn!(
                    "Modbus RX invalid frame: reason=too_short len={} data={=[u8]:x}",
                    bytes_read,
                    &rx_buf[..bytes_read]
                );
            }
            Err(RequestError::CrcMismatch) => {
                log_crc_mismatch(&rx_buf[..bytes_read]);
            }
            Err(RequestError::IllegalFunction) => {
                warn!(
                    "Modbus request handling failed: reason=illegal_function slave={} function={} frame={=[u8]:x}",
                    rx_buf.first().copied().unwrap_or(0),
                    rx_buf.get(1).copied().unwrap_or(0),
                    &rx_buf[..bytes_read]
                );
            }
            Err(RequestError::IllegalDataAddress) => {
                warn!(
                    "Modbus request handling failed: reason=illegal_data_address slave={} function={} start=0x{:04x} quantity={} frame={=[u8]:x}",
                    rx_buf.first().copied().unwrap_or(0),
                    rx_buf.get(1).copied().unwrap_or(0),
                    parse_start_address(&rx_buf[..bytes_read]),
                    parse_quantity(&rx_buf[..bytes_read]),
                    &rx_buf[..bytes_read]
                );
            }
            Err(RequestError::IllegalDataValue) => {
                warn!(
                    "Modbus request handling failed: reason=illegal_data_value slave={} function={} start=0x{:04x} quantity={} frame={=[u8]:x}",
                    rx_buf.first().copied().unwrap_or(0),
                    rx_buf.get(1).copied().unwrap_or(0),
                    parse_start_address(&rx_buf[..bytes_read]),
                    parse_quantity(&rx_buf[..bytes_read]),
                    &rx_buf[..bytes_read]
                );
            }
            Err(RequestError::ResponseBufferTooSmall) => {
                warn!("Modbus request handling failed: reason=response_buffer_too_small");
            }
            Err(RequestError::NotForThisSlave) => {}
        }
    }
}

fn log_crc_mismatch(frame: &[u8]) {
    let received_crc = if frame.len() >= 2 {
        u16::from_le_bytes([frame[frame.len() - 2], frame[frame.len() - 1]])
    } else {
        0
    };

    let calculated_crc = if frame.len() >= 2 {
        crc16(&frame[..frame.len() - 2])
    } else {
        0
    };

    warn!(
        "Modbus RX invalid frame: reason=crc_mismatch len={} slave={} function={} expected_crc=0x{:04x} received_crc=0x{:04x} start=0x{:04x} quantity={} frame={=[u8]:x}",
        frame.len(),
        frame.first().copied().unwrap_or(0),
        frame.get(1).copied().unwrap_or(READ_INPUT_REGISTERS),
        calculated_crc,
        received_crc,
        parse_start_address(frame),
        parse_quantity(frame),
        frame
    );
}

fn parse_start_address(frame: &[u8]) -> u16 {
    if frame.len() >= 4 {
        u16::from_be_bytes([frame[2], frame[3]])
    } else {
        0
    }
}

fn parse_quantity(frame: &[u8]) -> u16 {
    if frame.len() >= 6 {
        u16::from_be_bytes([frame[4], frame[5]])
    } else {
        0
    }
}
