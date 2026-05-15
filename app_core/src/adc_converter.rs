pub struct StoredValues {
    voltages: [u16; 4],
    currents: [u16; 4],
}

impl StoredValues {
    pub fn new() -> Self {
        Self {
            voltages: [0; 4],
            currents: [0; 4],
        }
    }

    pub fn update(&mut self, new_voltages: [u16; 4], new_currents: [u16; 4]) {
        self.voltages = new_voltages;
        self.currents = new_currents;
    }

    pub fn get_voltages(&self) -> &[u16; 4] {
        &self.voltages
    }

    pub fn get_currents(&self) -> &[u16; 4] {
        &self.currents
    }
}