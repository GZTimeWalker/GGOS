use super::uart16550::SerialPort;

const SERIAL_COM1_PORT: u16 = 0x3F8; // COM1

once_mutex!(pub SERIAL: SerialPort<SERIAL_COM1_PORT>);

pub fn init() {
    unsafe {
        init_SERIAL(SerialPort::new());
    }
    get_serial_for_sure().init();
    println!("{}", crate::get_ascii_header());
    println!("[+] Serial Initialized.");
}

guard_access_fn!(pub get_serial(SERIAL: SerialPort<SERIAL_COM1_PORT>));

pub fn backspace() {
    get_serial_for_sure().send(8);
}
