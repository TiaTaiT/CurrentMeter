// File: /app_firmware/src/hardware/sensors.rs
use embassy_stm32::adc::{Adc, AnyAdcChannel, SampleTime};
use embassy_stm32::peripherals::ADC1;

use app_core::hardware_traits::{SensorInterface};

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
        [c0, c1, c2, c3]
    }
    pub async fn read_voltages(&mut self) -> [u16; 4] {
        let v0 = self.adc.read(&mut self.voltages[0], SampleTime::CYCLES160_5).await;
        let v1 = self.adc.read(&mut self.voltages[1], SampleTime::CYCLES160_5).await;
        let v2 = self.adc.read(&mut self.voltages[2], SampleTime::CYCLES160_5).await;
        let v3 = self.adc.read(&mut self.voltages[3], SampleTime::CYCLES160_5).await;
        [v0, v1, v2, v3]
    }
}

impl SensorInterface for SystemSensor {
    async fn read_currents(&mut self) -> [u16; 4] { SystemSensor::read_currents(self).await }
    async fn read_voltages(&mut self) -> [u16; 4] { SystemSensor::read_voltages(self).await }
}