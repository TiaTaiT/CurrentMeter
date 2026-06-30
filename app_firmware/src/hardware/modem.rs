// File: /app_firmware/src/hardware/modem.rs
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::{Error as UartError, UartRx, UartTx};
use app_core::hardware_traits::{
    ModbusRxInterface,
    ModbusTxInterface,
    ModemControlInterface,
    PowerState
};

use crate::hardware::apply_state;

pub struct ModemRx(pub UartRx<'static, Async>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
pub enum ModbusRxError {
    Framing,
    Noise,
    Overrun,
    Parity,
    BufferTooLong,
    Other,
}

impl From<UartError> for ModbusRxError {
    fn from(value: UartError) -> Self {
        match value {
            UartError::Framing => Self::Framing,
            UartError::Noise => Self::Noise,
            UartError::Overrun => Self::Overrun,
            UartError::Parity => Self::Parity,
            UartError::BufferTooLong => Self::BufferTooLong,
            _ => Self::Other,
        }
    }
}

pub struct ModemTx(pub UartTx<'static, Async>);

pub struct ModemControl {
    pub(crate) enable: Output<'static>,
}

impl ModemControl {
    pub fn set_enable(&mut self, state: PowerState) {
        apply_state(&mut self.enable, state);
    }

    pub fn set_transmit_mode(&mut self) {
        self.set_enable(PowerState::On);
    }

    pub fn set_receive_mode(&mut self) {
        self.set_enable(PowerState::Off);
    }
}

impl ModemControlInterface for ModemControl {
    fn set_enable(&mut self, state: PowerState) {
        ModemControl::set_enable(self, state);
    }
}

impl ModbusTxInterface for ModemTx {
    async fn write(&mut self, buf: &[u8]) -> Result<(), ()> {
        self.0.write(buf).await.map_err(|_| ())
    }
}

impl ModbusRxInterface for ModemRx {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        self.0.read(buf).await.map_err(|_| ())?;
        Ok(buf.len())
    }

    async fn read_until_idle(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        self.0.read_until_idle(buf).await.map_err(|_| ())
    }
}

impl ModemRx {
    pub async fn read_until_idle_detailed(&mut self, buf: &mut [u8]) -> Result<usize, ModbusRxError> {
        self.0.read_until_idle(buf).await.map_err(ModbusRxError::from)
    }
}
