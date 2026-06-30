use core::{cell::Cell, default::Default};

unsafe impl Sync for StoredValues {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MeasurementSnapshot {
    pub voltages: [u16; 4],
    pub currents: [u16; 4],
}

impl MeasurementSnapshot {
    pub const fn new(voltages: [u16; 4], currents: [u16; 4]) -> Self {
        Self { voltages, currents }
    }

    pub const fn zeroed() -> Self {
        Self::new([0; 4], [0; 4])
    }

}

pub struct StoredValues {
    voltages: [Cell<u16>; 4],
    currents: [Cell<u16>; 4],
}

impl StoredValues {
    pub const fn new() -> Self {
        Self {
            voltages: [const { Cell::new(0) }; 4],
            currents: [const { Cell::new(0) }; 4],
        }
    }

    pub fn update(&self, new_voltages: [u16; 4], new_currents: [u16; 4]) {
        for (slot, value) in self.voltages.iter().zip(new_voltages) {
            slot.set(value);
        }

        for (slot, value) in self.currents.iter().zip(new_currents) {
            slot.set(value);
        }
    }

    pub fn snapshot(&self) -> MeasurementSnapshot {
        MeasurementSnapshot {
            voltages: self.get_voltages(),
            currents: self.get_currents(),
        }
    }

    pub fn get_voltages(&self) -> [u16; 4] {
        self.voltages.each_ref().map(Cell::get)
    }

    pub fn get_currents(&self) -> [u16; 4] {
        self.currents.each_ref().map(Cell::get)
    }
}

impl Default for StoredValues {
    fn default() -> Self {
        Self::new()
    }
}