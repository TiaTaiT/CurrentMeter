// File: /app_firmware/src/hardware/init.rs

use defmt::info;
use embassy_stm32::{Config, rcc::{AHBPrescaler, Sysclk}};

pub struct Hardware {
}

pub fn init() -> Hardware {
    let mut config = Config::default();
    
    // 1. Enable the 16 MHz High-Speed Internal (HSI) RC oscillator
    config.rcc.hsi = true; 
    
    // 2. Set the System Clock (SYSCLK) source to use the 16 MHz HSI
    config.rcc.sys = Sysclk::HSI;
    
    // 3. Divide the 16 MHz SYSCLK by 4 to get exactly a 4 MHz AHB/Core Clock
    config.rcc.ahb_pre = AHBPrescaler::DIV4;
    
    let p = embassy_stm32::init(config);
    
    // Since embassy calculates UART/ADC baudrates dynamically using its own 
    // internal clock tree solver, it will automatically know your core is 4 MHz. 
    // We can keep your constant here strictly for logging.
    info!("Hardware initialized!");

    Hardware {
    }
}