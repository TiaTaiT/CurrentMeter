// File: /app_firmware/src/hardware/leds.rs
use app_core::hardware_traits::{LedControl, PowerState};
use embassy_stm32::gpio::Output;

pub struct StatusLeds {
    pub(crate) sys_led: Output<'static>,
}

impl StatusLeds {
    pub fn set_sys_led(&mut self, state: PowerState) { apply_state(&mut self.sys_led, state); }
}

impl LedControl for StatusLeds {
    fn set_system(&mut self, state: PowerState) { StatusLeds::set_sys_led(self, state); }
}

fn apply_state(pin: &mut Output<'static>, state: PowerState) {
    match state {
        PowerState::On => pin.set_high(),
        PowerState::Off => pin.set_low(),
    }
}
