// File: /app_firmware/src/hardware/modem.rs
use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::{UartRx, UartTx};
use app_core::hardware_traits::{
    ModbusRxInterface,
    ModbusTxInterface,
    ModemControlInterface,
    PowerState
};

use crate::hardware::apply_state;

pub struct ModemRx(pub UartRx<'static, Async>);

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
