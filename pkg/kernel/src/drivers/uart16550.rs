/// reference: https://docs.rs/uart_16550
/// reference: http://byterunner.com/16550.html
/// reference: http://www.larvierinehart.com/serial/serialadc/serial.htm
/// reference: https://wiki.osdev.org/Serial_Ports

use core::fmt;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};
use bitflags::bitflags;

macro_rules! wait_for {
    ($cond:expr) => {
        while !$cond {
            core::hint::spin_loop()
        }
    };
}

bitflags! {
    /// Line status flags
    struct LineStsFlags: u8 {
        const INPUT_FULL = 1;
        // 1 to 4 unknown
        const OUTPUT_EMPTY = 1 << 5;
        // 6 and 7 unknown
    }
}

/// A port-mapped UART.
#[cfg_attr(docsrs, doc(cfg(target_arch = "x86_64")))]
pub struct SerialPort {
    /// - ransmit Holding Register (write)
    /// - receive Holding Register (read)
    data: Port<u8>,
    /// Interrupt Enable Register
    /// - bit 0: receive holding register interrupt
    /// - bit 1: transmit holding register interrupt
    /// - bit 2: receive line status interrupt
    /// - bit 3: modem status interrupt
    int_en: PortWriteOnly<u8>,
    /// FIFO Control Register
    fifo_ctrl: PortWriteOnly<u8>,
    /// Line Control Register
    line_ctrl: PortWriteOnly<u8>,
    /// Modem Control Register
    modem_ctrl: PortWriteOnly<u8>,
    /// Line Status Register
    line_sts: PortReadOnly<u8>,
}

impl SerialPort {
    /// Creates a new serial port interface on the given I/O port.
    ///
    /// This function is unsafe because the caller must ensure that the given base address
    /// really points to a serial port device.
    pub const unsafe fn new(base: u16) -> Self {
        Self {
            data: Port::new(base),
            int_en: PortWriteOnly::new(base + 1),
            fifo_ctrl: PortWriteOnly::new(base + 2),
            line_ctrl: PortWriteOnly::new(base + 3),
            modem_ctrl: PortWriteOnly::new(base + 4),
            line_sts: PortReadOnly::new(base + 5),
        }
    }

    /// Initializes the serial port.
    ///
    /// The default configuration of [38400/8-N-1](https://en.wikipedia.org/wiki/8-N-1) is used.
    pub fn init(&mut self) {
        unsafe {
            // Disable interrupts
            self.int_en.write(0x00);

            // Enable DLAB
            self.line_ctrl.write(0x80);

            // Set maximum speed to 38400 bps by configuring DLL and DLM
            // > LSB of Divisor Latch when Enabled
            self.data.write(0b00000011);
            // > MSB of Divisor Latch when Enabled
            self.int_en.write(0b00000000);

            // Disable DLAB and set data word length to 8 bits
            self.line_ctrl.write(0b00000011);

            // Enable FIFO, clear TX/RX queues and
            // set interrupt watermark at 1 bytes
            self.fifo_ctrl.write(0b00000111);

            // Mark data terminal ready, signal request to send
            // and enable auxilliary output #2 (used as interrupt line for CPU)
            self.modem_ctrl.write(0b00001011);

            // Enable interrupts
            self.int_en.write(0b00000001);
        }
    }

    fn line_sts(&mut self) -> LineStsFlags {
        unsafe { LineStsFlags::from_bits_truncate(self.line_sts.read()) }
    }

    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        unsafe {
            match data {
                8 | 0x7F => {
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self.data.write(8);
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self.data.write(b' ');
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self.data.write(8)
                }
                _ => {
                    wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
                    self.data.write(data);
                }
            }
        }
    }

    /// Sends a raw byte on the serial port, intended for binary data.
    pub fn send_raw(&mut self, data: u8) {
        unsafe {
            wait_for!(self.line_sts().contains(LineStsFlags::OUTPUT_EMPTY));
            self.data.write(data);
        }
    }

    /// Receives a byte on the serial port.
    pub fn receive(&mut self) -> u8 {
        unsafe {
            wait_for!(self.line_sts().contains(LineStsFlags::INPUT_FULL));
            self.data.read()
        }
    }

    /// Receives a byte on the serial port no wait.
    pub fn receive_no_wait(&mut self) -> Option<u8> {
        unsafe {
            if self.line_sts().contains(LineStsFlags::INPUT_FULL) {
                return Some(self.data.read());
            }
            None
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
