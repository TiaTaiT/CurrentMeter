use core::cell::{RefCell};

pub struct StoredValues {
    voltages: RefCell<[u16; 4]>,
    currents: RefCell<[u16; 4]>,
}

impl StoredValues {
    pub fn new() -> Self {
        Self {
            voltages: RefCell::new([0; 4]),
            currents: RefCell::new([0; 4]),
        }
    }

    pub fn update(&self, new_voltages: [u16; 4], new_currents: [u16; 4]) {
        *self.voltages.borrow_mut() = new_voltages;
        *self.currents.borrow_mut() = new_currents;
    }

    pub fn get_voltages(&self) -> [u16; 4] {
        *self.voltages.borrow()
    }

    pub fn get_currents(&self) -> [u16; 4] {
        *self.currents.borrow()
    }
}