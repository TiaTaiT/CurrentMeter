// File: /app_firmware/src/hardware/init.rs

use defmt::info;
use embassy_stm32::{
    Config,
    adc::{self, Adc, AdcChannel},
    bind_interrupts,
    dma,
    gpio::{Level, Output, Speed},
    peripherals::ADC1,
    rcc::{AHBPrescaler, Sysclk},
    usart
};

use crate::hardware::{StatusLeds, SystemSensor, modem::{ModemControl, ModemRx, ModemTx}};
use embassy_stm32::usart::{Config as UartConfig, Uart};

bind_interrupts!(struct Irqs {
    ADC1_COMP => adc::InterruptHandler<ADC1>;
    USART1    => usart::InterruptHandler<embassy_stm32::peripherals::USART1>;
    DMA1_CHANNEL2_3 => dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH2>, dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH3>;
    DMA1_CHANNEL4_5_6_7 => dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH4>, dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH5>;
});

pub struct Hardware {
    pub leds: StatusLeds,
    pub sensors: SystemSensor,
    pub modem_tx: ModemTx,
    pub modem_rx: ModemRx,
    pub modem_ctrl: ModemControl,
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

    let leds = StatusLeds {
        sys_led: Output::new(p.PB12, Level::Low, Speed::Low),
    };

    let adc = Adc::new(p.ADC1, Irqs);
    let sensors = SystemSensor {
        currents: [p.PA4.degrade_adc(), p.PA5.degrade_adc(), p.PA6.degrade_adc(), p.PA7.degrade_adc()],
        voltages: [p.PA2.degrade_adc(), p.PA3.degrade_adc(), p.PB0.degrade_adc(), p.PB1.degrade_adc()],
        adc,
    };

    let mut config_u1 = UartConfig::default();
    config_u1.baudrate = 9600;
    let (modem_tx, modem_rx) = Uart::new(
        p.USART1,
        p.PA10,
        p.PA9,
        p.DMA1_CH2,
        p.DMA1_CH3,
        Irqs,
        config_u1,
    ).unwrap().split();

    let modem_tx = ModemTx(modem_tx);
    let modem_rx = ModemRx(modem_rx);

    let modem_ctrl = ModemControl {
        enable: Output::new(p.PB4, Level::High, Speed::Low),
    };

    Hardware {
        leds,
        sensors,
        modem_tx,
        modem_rx,
        modem_ctrl,
    }
}