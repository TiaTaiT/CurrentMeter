use core::{option::Option, result::Result};

use crate::adc_converter::MeasurementSnapshot;

pub const READ_INPUT_REGISTERS: u8 = 0x04;
pub const EXCEPTION_OFFSET: u8 = 0x80;
pub const ILLEGAL_FUNCTION: u8 = 0x01;
pub const ILLEGAL_DATA_ADDRESS: u8 = 0x02;
pub const ILLEGAL_DATA_VALUE: u8 = 0x03;
pub const MAX_READ_REGISTERS: u16 = 125;
const REQUEST_LEN: usize = 8;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ReadInputRegistersRequest {
    pub slave: u8,
    pub start_address: u16,
    pub quantity: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RequestError {
    FrameTooShort,
    CrcMismatch,
    NotForThisSlave,
    IllegalFunction,
    IllegalDataAddress,
    IllegalDataValue,
    ResponseBufferTooSmall,
}

impl RequestError {
    pub const fn exception_code(self) -> Option<u8> {
        match self {
            Self::IllegalFunction => Some(ILLEGAL_FUNCTION),
            Self::IllegalDataAddress => Some(ILLEGAL_DATA_ADDRESS),
            Self::IllegalDataValue => Some(ILLEGAL_DATA_VALUE),
            _ => None,
        }
    }
}

pub fn crc16(data: &[u8]) -> u16 {
    let mut crc = 0xFFFFu16;

    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 0x0001 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }

    crc
}

pub fn parse_read_input_registers_request(
    frame: &[u8],
    expected_slave: u8,
) -> Result<ReadInputRegistersRequest, RequestError> {
    if frame.len() < REQUEST_LEN {
        return Err(RequestError::FrameTooShort);
    }

    let payload = &frame[..frame.len() - 2];
    let received_crc = u16::from_le_bytes([frame[frame.len() - 2], frame[frame.len() - 1]]);
    if crc16(payload) != received_crc {
        return Err(RequestError::CrcMismatch);
    }

    if frame[0] != expected_slave {
        return Err(RequestError::NotForThisSlave);
    }

    if frame[1] != READ_INPUT_REGISTERS {
        return Err(RequestError::IllegalFunction);
    }

    let start_address = u16::from_be_bytes([frame[2], frame[3]]);
    let quantity = u16::from_be_bytes([frame[4], frame[5]]);

    if quantity == 0 || quantity > MAX_READ_REGISTERS {
        return Err(RequestError::IllegalDataValue);
    }

    let end_address = start_address
        .checked_add(quantity - 1)
        .ok_or(RequestError::IllegalDataAddress)?;

    if (start_address..=end_address)
        .any(|address| snapshot_address_unavailable(address))
    {
        return Err(RequestError::IllegalDataAddress);
    }

    Ok(ReadInputRegistersRequest {
        slave: expected_slave,
        start_address,
        quantity,
    })
}

pub fn get_register_value(snapshot: &MeasurementSnapshot, address: u16) -> Option<u16> {
    match address {
        0x1000..=0x1003 => Some(snapshot.voltages[(address - 0x1000) as usize]),
        0x1004..=0x1007 => Some(snapshot.currents[(address - 0x1004) as usize]),
        _ => None,
    }
}

pub fn build_read_input_registers_response(
    request: ReadInputRegistersRequest,
    snapshot: &MeasurementSnapshot,
    response: &mut [u8],
) -> Result<usize, RequestError> {
    let byte_count = request.quantity as usize * 2;
    let frame_len = 3 + byte_count + 2;
    if response.len() < frame_len {
        return Err(RequestError::ResponseBufferTooSmall);
    }

    response[0] = request.slave;
    response[1] = READ_INPUT_REGISTERS;
    response[2] = byte_count as u8;

    for index in 0..request.quantity as usize {
        let address = request.start_address + index as u16;
        let value = get_register_value(snapshot, address)
            .ok_or(RequestError::IllegalDataAddress)?;
        let bytes = value.to_be_bytes();
        let offset = 3 + index * 2;
        response[offset] = bytes[0];
        response[offset + 1] = bytes[1];
    }

    let crc = crc16(&response[..frame_len - 2]).to_le_bytes();
    response[frame_len - 2] = crc[0];
    response[frame_len - 1] = crc[1];

    Ok(frame_len)
}

pub fn build_exception_response(
    slave: u8,
    function: u8,
    exception_code: u8,
    response: &mut [u8],
) -> Result<usize, RequestError> {
    if response.len() < 5 {
        return Err(RequestError::ResponseBufferTooSmall);
    }

    response[0] = slave;
    response[1] = function | EXCEPTION_OFFSET;
    response[2] = exception_code;

    let crc = crc16(&response[..3]).to_le_bytes();
    response[3] = crc[0];
    response[4] = crc[1];

    Ok(5)
}

pub fn handle_request(
    frame: &[u8],
    expected_slave: u8,
    snapshot: &MeasurementSnapshot,
    response: &mut [u8],
) -> Result<Option<usize>, RequestError> {
    match parse_read_input_registers_request(frame, expected_slave) {
        Ok(request) => build_read_input_registers_response(request, snapshot, response).map(Some),
        Err(RequestError::NotForThisSlave) => Ok(None),
        Err(error) => {
            if let Some(exception_code) = error.exception_code() {
                let function = frame.get(1).copied().unwrap_or(READ_INPUT_REGISTERS);
                build_exception_response(expected_slave, function, exception_code, response).map(Some)
            } else {
                Err(error)
            }
        }
    }
}

fn snapshot_address_unavailable(address: u16) -> bool {
    !(0x1000..=0x1007).contains(&address)
}