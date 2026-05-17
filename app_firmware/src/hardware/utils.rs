// File: //app_firmware/src/hardware/utils.rs

use app_core::hardware_traits::PowerState;
use embassy_stm32::gpio::Output;

pub fn apply_state(pin: &mut Output<'static>, state: PowerState) {
    match state {
        PowerState::On => pin.set_high(),
        PowerState::Off => pin.set_low(),
    }
}