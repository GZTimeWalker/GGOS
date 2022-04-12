#[warn(dead_code)]

use uart_16550::SerialPort;

const SERIAL_IO_PORT: u16 = 0x3F8;

once_mutex!(pub SERIAL: SerialPort);

pub unsafe fn init() {
    init_SERIAL(SerialPort::new(SERIAL_IO_PORT));
}

guard_access_fn!(pub get_serial(SERIAL: SerialPort));

pub fn backspace() {
    let mut serial = get_serial_for_sure();
    serial.send(b'\x08');
    serial.send(b' ');
    serial.send(b'\x08');
}
