#![no_std]
#![no_main]

use app_core::{CurrentMeter, MeterConfig};
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_time::{Duration, Timer};
use defmt::*;
use defmt_rtt as _;
use panic_halt as _;

critical_section::set_impl!(CriticalSectionImpl);

struct CriticalSectionImpl;

unsafe impl critical_section::Impl for CriticalSectionImpl {
    unsafe fn acquire() -> critical_section::RawRestoreState {
        cortex_m::interrupt::disable();
        0
    }

    unsafe fn release(_token: critical_section::RawRestoreState) {
        unsafe { cortex_m::interrupt::enable() };
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());
    let _ = p;

    let meter = CurrentMeter::new(MeterConfig {
        offset_milliamps: 0,
        scale_microamps_per_lsb: 805,
    });

    loop {
        let raw = 2048u16;
        let _sample = meter.from_adc(raw);
        info!("Sample");
        Timer::after(Duration::from_millis(500)).await;
    }
}
