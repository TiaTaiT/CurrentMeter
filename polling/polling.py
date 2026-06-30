#!/usr/bin/env python3
import sys
import time
import argparse
import struct
import serial

# Modbus Configuration matching the STM32 firmware constants
SLAVE_ADDRESS = 7
READ_INPUT_REGISTERS = 0x04
START_ADDRESS = 0x1000
QUANTITY = 8  # 4 voltages + 4 currents

# Exception Codes
EXCEPTION_CODES = {
    1: "Illegal Function",
    2: "Illegal Data Address",
    3: "Illegal Data Value",
}

def calculate_crc16(data: bytes) -> int:
    """Calculates the standard Modbus RTU CRC-16."""
    crc = 0xFFFF
    for byte in data:
        crc ^= byte
        for _ in range(8):
            if crc & 0x0001:
                crc = (crc >> 1) ^ 0xA001
            else:
                crc >>= 1
    return crc

def build_read_request(slave: int, start_addr: int, quantity: int) -> bytes:
    """Constructs a Modbus RTU Read Input Registers request frame."""
    frame = struct.pack(">BBHH", slave, READ_INPUT_REGISTERS, start_addr, quantity)
    crc = calculate_crc16(frame)
    # CRC is appended as little-endian
    frame += struct.pack("<H", crc)
    return frame

def parse_response(response: bytes, expected_qty: int):
    """
    Parses and validates the Modbus RTU response frame.
    Returns a tuple of (voltages, currents) or raises an error.
    """
    if len(response) < 5:
        raise ValueError("Response is too short")

    # Verify CRC
    payload = response[:-2]
    received_crc = struct.unpack("<H", response[-2:])[0]
    calculated_crc = calculate_crc16(payload)
    if received_crc != calculated_crc:
        raise ValueError(f"CRC Mismatch: Calculated 0x{calculated_crc:04X}, received 0x{received_crc:04X}")

    slave, function = response[0], response[1]
    
    # Handle Modbus Exceptions
    if function & 0x80:
        exception_code = response[2]
        error_msg = EXCEPTION_CODES.get(exception_code, f"Unknown code {exception_code}")
        raise ValueError(f"Modbus Exception: {error_msg}")

    if function != READ_INPUT_REGISTERS:
        raise ValueError(f"Unexpected Function Code: {function}")

    byte_count = response[2]
    expected_bytes = expected_qty * 2
    if byte_count != expected_bytes or len(response) != 3 + expected_bytes + 2:
        raise ValueError(f"Unexpected data length. Expected {expected_bytes} data bytes, got {byte_count}")

    # Extract register values (big-endian u16s)
    registers = []
    for i in range(expected_qty):
        offset = 3 + (i * 2)
        val = struct.unpack(">H", response[offset:offset+2])[0]
        registers.append(val)

    # First 4 are voltages, next 4 are currents
    voltages = registers[0:4]
    currents = registers[4:8]
    return voltages, currents

def main():
    parser = argparse.ArgumentParser(description="Desktop Polling Script for STM32 Modbus Slave")
    parser.add_argument("-p", "--port", required=True, help="Serial port (e.g., COM3 on Windows or /dev/ttyUSB0 on Linux)")
    parser.add_argument("-b", "--baud", type=int, default=9600, help="Baud rate (default: 9600)")
    parser.add_argument("-i", "--interval", type=float, default=1.0, help="Polling interval in seconds (default: 1.0)")
    args = parser.parse_args()

    print(f"Connecting to {args.port} at {args.baud} baud...")
    
    try:
        # 1-second timeout is typically sufficient for 9600 baud
        ser = serial.Serial(
            port=args.port,
            baudrate=args.baud,
            bytesize=serial.EIGHTBITS,
            parity=serial.PARITY_NONE,
            stopbits=serial.STOPBITS_ONE,
            timeout=1.0
        )
    except serial.SerialException as e:
        print(f"Error opening serial port: {e}")
        sys.exit(1)

    request_frame = build_read_request(SLAVE_ADDRESS, START_ADDRESS, QUANTITY)
    
    # Expected frame size: 1 (slave) + 1 (function) + 1 (byte count) + 16 (data) + 2 (crc) = 21 bytes
    expected_response_len = 3 + (QUANTITY * 2) + 2

    print(f"Polling Modbus Slave {SLAVE_ADDRESS} (Registers 0x1000 - 0x1007) every {args.interval}s...")
    print("Press Ctrl+C to stop.\n")

    try:
        while True:
            # Flush existing buffers to prevent stale frames
            ser.reset_input_buffer()
            ser.reset_output_buffer()

            # Transmit Request
            ser.write(request_frame)

            # Read Response
            response = ser.read(expected_response_len)

            if not response:
                print("[Warning] Timeout: No response received from target device.")
            else:
                try:
                    voltages, currents = parse_response(response, QUANTITY)
                    print(f"Voltages (0x1000-0x1003): {voltages} | Currents (0x1004-0x1007): {currents}")
                except ValueError as err:
                    print(f"[Error] Failed to parse response: {err}")
                    print(f"Raw received bytes: {response.hex(' ')}")

            time.sleep(args.interval)

    except KeyboardInterrupt:
        print("\nPolling stopped by user.")
    finally:
        ser.close()
        print("Serial port closed.")

if __name__ == "__main__":
    main()