use super::{inb, outb};
use core::fmt::Write;

const COM1: u16 = 0x3f8;
// when DLAB = 0
const INTERRUPT_REG: u16 = 1;
const LINE_CTRL_REG: u16 = 3;
// when DLAB = 1
const DIVISOR_LO_REG: u16 = 0;
// when DLAB = 1
const DIVISOR_HI_REG: u16 = 1;
const INTERRUPT_IDENT_REG: u16 = 2;
const MODEM_CTRL_REG: u16 = 4;
// when DLAB = 0
const DATA_REG: u16 = 0;

const LINE_STATUS_REG: u16 = 5;

const TEST_MAGIC: u8 = 0xae;

pub struct Serial;

impl Serial {
    pub fn init() -> bool {
        unsafe {
            outb(COM1 + INTERRUPT_REG, 0x00);
            outb(COM1 + LINE_CTRL_REG, 0x80);
            outb(COM1 + DIVISOR_LO_REG, 0x03);
            outb(COM1 + DIVISOR_HI_REG, 0x00);
            outb(COM1 + LINE_CTRL_REG, 0x03);
            outb(COM1 + INTERRUPT_IDENT_REG, 0xc7);
            outb(COM1 + MODEM_CTRL_REG, 0x0b);
            outb(COM1 + MODEM_CTRL_REG, 0x1e);
            outb(COM1 + DATA_REG, TEST_MAGIC);

            if inb(COM1 + DATA_REG) != TEST_MAGIC {
                return false;
            }

            outb(COM1 + MODEM_CTRL_REG, 0x0f);

            true
        }
    }

    fn is_transmit_empty() -> bool {
        (unsafe { inb(COM1 + LINE_STATUS_REG) } & 0x20) != 0
    }

    fn write_byte(byte: u8) {
        while !Self::is_transmit_empty() {}
        unsafe {
            outb(COM1 + DATA_REG, byte);
        }
    }
}

impl Write for Serial {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            Serial::write_byte(b);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! kprintln {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut s = Serial;
        let _ = writeln!(s, $($arg)*);
    }};
}
