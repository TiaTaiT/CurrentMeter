// File: /app_core/src/lib.rs

// This file only contains the public API of the app_core crate.
//It re-exports modules and items that are meant to be used by other crates, such as app_firmware.
// The internal implementation details of the modules are hidden from external users.
// Don't put any implementation code here; instead, place it in the respective modules (e.g., adc_converter.rs).
#![no_std]

pub mod hardware_traits;
pub mod adc_converter;
pub mod modbus;
#[cfg(test)]
pub mod tests;
