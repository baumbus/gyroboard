#![no_std]
#![no_main]
#![feature(ascii_char)]
#![allow(unused_imports)]

use arduino_hal::{hal::i2c, prelude::*};
use gyroboard::liquid_crystal_i2c::LcdI2c;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    //let dp: arduino_hal::Peripherals = arduino_hal::Peripherals::take().unwrap();
    //let pins: arduino_hal::Pins = arduino_hal::pins!(dp);
    //let _serial = arduino_hal::default_serial!(dp, pins, 57600);

    // let _i2c = arduino_hal::I2c::new(
    // dp.TWI,
    // pins.a4.into_pull_up_input(),
    // pins.a5.into_pull_up_input(),
    // 50000,
    // );

    let mut display = LcdI2c::new(0x27, false, 16, 2, 0);

    display.init();
    display.set_backlight(true);
    display.set_blink(false);
    display.move_cursor(0, 0);
    display.set_cursor(false);

    display.println("Hello".as_ascii().unwrap().as_bytes());
    display.println("Penis".as_ascii().unwrap().as_bytes());

    loop {
        arduino_hal::delay_ms(1000);
        display.println("Hello".as_ascii().unwrap().as_bytes());
        display.move_cursor(0, 0);
        arduino_hal::delay_ms(1000);
        display.println("Penis".as_ascii().unwrap().as_bytes());
        display.move_cursor(0, 0);
    }
}
