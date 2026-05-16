#[cfg(test)]
mod tests {
use crate::adc_converter::StoredValues;

use super::*;

    #[test]
    fn test_new_initializes_with_zeros() {
        let stored = StoredValues::new();
        
        // Assert that the initial state is all zeros
        assert_eq!(stored.get_voltages(), [0, 0, 0, 0]);
        assert_eq!(stored.get_currents(), [0, 0, 0, 0]);
    }

    #[test]
    fn test_update_modifies_values() {
        // Note: `stored` does not need to be `mut` because `update` takes `&self`
        // and uses interior mutability via `RefCell`.
        let stored = StoredValues::new();
        
        let new_voltages = [100, 200, 300, 400];
        let new_currents = [10, 20, 30, 40];
        
        stored.update(new_voltages, new_currents);
        
        assert_eq!(stored.get_voltages(), new_voltages);
        assert_eq!(stored.get_currents(), new_currents);
    }

    #[test]
    fn test_multiple_updates() {
        let stored = StoredValues::new();
        
        // First update
        stored.update([1, 2, 3, 4], [5, 6, 7, 8]);
        assert_eq!(stored.get_voltages(), [1, 2, 3, 4]);
        assert_eq!(stored.get_currents(), [5, 6, 7, 8]);
        
        // Second update, to ensure previous data is overwritten properly
        stored.update([11, 22, 33, 44], [55, 66, 77, 88]);
        assert_eq!(stored.get_voltages(), [11, 22, 33, 44]);
        assert_eq!(stored.get_currents(), [55, 66, 77, 88]);
    }

    #[test]
    fn test_update_with_max_values() {
        let stored = StoredValues::new();
        
        // u16 boundary testing
        let max_vals = [u16::MAX; 4];
        stored.update(max_vals, max_vals);
        
        assert_eq!(stored.get_voltages(), [65535, 65535, 65535, 65535]);
        assert_eq!(stored.get_currents(), [65535, 65535, 65535, 65535]);
    }
}