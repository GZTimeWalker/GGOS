use super::uart16550::SerialPort;

const SERIAL_IO_PORT: u16 = 0x3F8; // COM1

once_mutex!(pub SERIAL: SerialPort);

pub fn init() {
    unsafe {
        init_SERIAL(SerialPort::new(SERIAL_IO_PORT));
    }
    get_serial_for_sure().init();
    println!("{}", crate::get_ascii_header());
}

guard_access_fn!(pub get_serial(SERIAL: SerialPort));

pub fn backspace() {
    get_serial_for_sure().send(8);
}
