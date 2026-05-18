#[cfg(test)]
mod tests {
    use crate::adc_converter::MeasurementSnapshot;
    use crate::constants::MODBUS_SLAVE_ADDR;
use crate::modbus::{
        ILLEGAL_DATA_ADDRESS, ILLEGAL_FUNCTION, READ_INPUT_REGISTERS, build_exception_response,
        crc16, handle_request, parse_read_input_registers_request,
    };
    use core::assert_eq;

    fn request_frame(slave: u8, function: u8, address: u16, quantity: u16) -> [u8; 8] {
        let mut frame = [0u8; 8];
        frame[0] = slave;
        frame[1] = function;
        frame[2..4].copy_from_slice(&address.to_be_bytes());
        frame[4..6].copy_from_slice(&quantity.to_be_bytes());
        let crc = crc16(&frame[..6]).to_le_bytes();
        frame[6] = crc[0];
        frame[7] = crc[1];
        frame
    }

    fn snapshot() -> MeasurementSnapshot {
        MeasurementSnapshot::new([120, 121, 122, 123], [220, 221, 222, 223])
    }

    #[test]
    fn parse_valid_read_input_registers_request() {
        let frame = request_frame(MODBUS_SLAVE_ADDR, READ_INPUT_REGISTERS, 0x1000, 2);

        let request = parse_read_input_registers_request(&frame, MODBUS_SLAVE_ADDR).unwrap();

        assert_eq!(request.slave, MODBUS_SLAVE_ADDR);
        assert_eq!(request.start_address, 0x1000);
        assert_eq!(request.quantity, 2);
    }

    #[test]
    fn handle_request_returns_input_register_values() {
        let frame = request_frame(MODBUS_SLAVE_ADDR, READ_INPUT_REGISTERS, 0x1000, 4);
        let mut response = [0u8; 64];

        let len = handle_request(&frame, MODBUS_SLAVE_ADDR, &snapshot(), &mut response)
            .unwrap()
            .unwrap();

        assert_eq!(len, 13);
        assert_eq!(response[0], MODBUS_SLAVE_ADDR);
        assert_eq!(response[1], READ_INPUT_REGISTERS);
        assert_eq!(response[2], 8);
        assert_eq!(&response[3..11], &[0, 120, 0, 121, 0, 122, 0, 123]);
    }

    #[test]
    fn handle_request_reads_across_voltage_current_boundary() {
        let frame = request_frame(MODBUS_SLAVE_ADDR, READ_INPUT_REGISTERS, 0x1002, 4);
        let mut response = [0u8; 64];

        let len = handle_request(&frame, MODBUS_SLAVE_ADDR, &snapshot(), &mut response)
            .unwrap()
            .unwrap();

        assert_eq!(len, 13);
        assert_eq!(&response[3..11], &[0, 122, 0, 123, 0, 220, 0, 221]);
    }

    #[test]
    fn handle_request_ignores_other_slave_address() {
        let wrong_addr = MODBUS_SLAVE_ADDR + 1;
        let frame = request_frame(wrong_addr, READ_INPUT_REGISTERS, 0x1000, 1);
        let mut response = [0u8; 64];

        let result = handle_request(&frame, MODBUS_SLAVE_ADDR, &snapshot(), &mut response).unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn handle_request_returns_illegal_function_exception() {
        let frame = request_frame(MODBUS_SLAVE_ADDR, 0x03, 0x1000, 1);
        let mut response = [0u8; 64];

        let len = handle_request(&frame, MODBUS_SLAVE_ADDR, &snapshot(), &mut response)
            .unwrap()
            .unwrap();

        assert_eq!(len, 5);
        assert_eq!(response[0], MODBUS_SLAVE_ADDR);
        assert_eq!(response[1], 0x83);
        assert_eq!(response[2], ILLEGAL_FUNCTION);
    }

    #[test]
    fn handle_request_returns_illegal_data_address_exception() {
        let frame = request_frame(MODBUS_SLAVE_ADDR, READ_INPUT_REGISTERS, 0x1007, 2);
        let mut response = [0u8; 64];

        let len = handle_request(&frame, MODBUS_SLAVE_ADDR, &snapshot(), &mut response)
            .unwrap()
            .unwrap();

        assert_eq!(len, 5);
        assert_eq!(response[1], READ_INPUT_REGISTERS | 0x80);
        assert_eq!(response[2], ILLEGAL_DATA_ADDRESS);
    }

    #[test]
    fn build_exception_response_appends_crc() {
        let mut response = [0u8; 5];
        let len = build_exception_response(MODBUS_SLAVE_ADDR, READ_INPUT_REGISTERS, ILLEGAL_DATA_ADDRESS, &mut response)
            .unwrap();

        assert_eq!(len, 5);
        let expected_crc = crc16(&response[..3]).to_le_bytes();
        assert_eq!([response[3], response[4]], expected_crc);
    }
}