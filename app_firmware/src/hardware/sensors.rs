// File: /app_firmware/src/hardware/sensors.rs
use embassy_stm32::adc::{Adc, AnyAdcChannel, SampleTime};
use embassy_stm32::peripherals::ADC1;

use app_core::hardware_traits::{SensorInterface};

const CURRENT_DIVIDER_1: f32 = 59.83;
const CURRENT_DIVIDER_2: f32 = 6.0;
const CURRENT_DIVIDER_3: f32 = 5.95;
const CURRENT_DIVIDER_4: f32 = 6.07;

const VOLTAGE_DIVIDER_1: f32 = 0.11246;
const VOLTAGE_DIVIDER_2: f32 = 0.1112;
const VOLTAGE_DIVIDER_3: f32 = 0.1112;
const VOLTAGE_DIVIDER_4: f32 = 0.1127;

pub struct SystemSensor {
    pub(crate) currents: [AnyAdcChannel<'static, ADC1>; 4],
    pub(crate) voltages: [AnyAdcChannel<'static, ADC1>; 4],
    pub(crate) adc: Adc<'static, ADC1>,
}

impl SystemSensor {
    pub async fn read_currents(&mut self) -> [u16; 4] {
        let c0 = self.adc.read(&mut self.currents[0], SampleTime::CYCLES160_5).await;
        let c1 = self.adc.read(&mut self.currents[1], SampleTime::CYCLES160_5).await;
        let c2 = self.adc.read(&mut self.currents[2], SampleTime::CYCLES160_5).await;
        let c3 = self.adc.read(&mut self.currents[3], SampleTime::CYCLES160_5).await;
        let current1 = libm::roundf((c1 as f32) / CURRENT_DIVIDER_1) as u16;
        let current2 = libm::roundf((c2 as f32) / CURRENT_DIVIDER_2) as u16;
        let current3 = libm::roundf((c3 as f32) / CURRENT_DIVIDER_3) as u16;
        let current4 = libm::roundf((c0 as f32) / CURRENT_DIVIDER_4) as u16;
        [current1, current2, current3, current4]
    }
    pub async fn read_voltages(&mut self) -> [u16; 4] {
        let v0 = self.adc.read(&mut self.voltages[0], SampleTime::CYCLES160_5).await;
        let v1 = self.adc.read(&mut self.voltages[1], SampleTime::CYCLES160_5).await;
        let v2 = self.adc.read(&mut self.voltages[2], SampleTime::CYCLES160_5).await;
        let v3 = self.adc.read(&mut self.voltages[3], SampleTime::CYCLES160_5).await;
        let voltage1 = libm::roundf((v0 as f32) / VOLTAGE_DIVIDER_1) as u16;
        let voltage2 = libm::roundf((v3 as f32) / VOLTAGE_DIVIDER_2) as u16;
        let voltage3 = libm::roundf((v2 as f32) / VOLTAGE_DIVIDER_3) as u16;
        let voltage4 = libm::roundf((v1 as f32) / VOLTAGE_DIVIDER_4) as u16;
        [voltage1, voltage2, voltage3, voltage4]
    }
}

impl SensorInterface for SystemSensor {
    async fn read_currents(&mut self) -> [u16; 4] { SystemSensor::read_currents(self).await }
    async fn read_voltages(&mut self) -> [u16; 4] { SystemSensor::read_voltages(self).await }
}