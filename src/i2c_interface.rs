use arduino_hal::prelude::*;

pub struct I2cInterface {
    i2c: Option<I2c<Atmega, TWI, Pin<Input, PC4>, Pin<Input, PC5>, MHz16>>,
}
