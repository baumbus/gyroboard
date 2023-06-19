#![no_std]
#![no_main]
#![feature(ascii_char)]

use arduino_hal::prelude::*;
use gyroboard::liquid_crystal_i2c::LcdI2c;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let mut display = LcdI2c::new(0x27, false, 16, 2, 0);

    display.init();
    display.set_backlight(true);
    display.set_blink(false);
    display.move_cursor(0, 0);
    display.set_cursor(false);

    display.println("Hello".as_ascii().unwrap().as_bytes());
    display.println("World".as_ascii().unwrap().as_bytes());

    loop {
        arduino_hal::delay_ms(1000);
    }
}
