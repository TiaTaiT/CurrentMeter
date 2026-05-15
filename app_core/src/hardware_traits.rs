// File: //app_firmware/src/hardware/init.rs
#![allow(async_fn_in_trait)]

#[derive(Copy, Clone, PartialEq,Debug)]
pub enum PowerState {
    On,
    Off,
}

pub trait LedControl {
    fn set_system(&mut self, state: PowerState);
}
