#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use gyroboard::liquid_crystal_i2c::LcdI2c;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let mut display = LcdI2c::new(0x27, false, 16, 2, 0);

    display.init();
    display.no_backlight();

    loop {
        arduino_hal::delay_ms(1000);
    }
}
