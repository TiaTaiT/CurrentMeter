// File: //app_firmware/src/hardware/init.rs
#![allow(async_fn_in_trait)]
use embassy_stm32::gpio::Output;

#[derive(Copy, Clone, PartialEq,Debug)]
pub enum PowerState {
    On,
    Off,
}

pub trait LedInterface {
    fn set_system(&mut self, state: PowerState);
}

pub(crate) fn apply_state(pin: &mut Output<'static>, state: PowerState) {
    match state {
        PowerState::On => pin.set_high(),
        PowerState::Off => pin.set_low(),
    }
}