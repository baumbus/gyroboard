#![no_std]

use arduino_hal::prelude::*;

// commands
const LCD_CLEARDISPLAY: u8 = 0x01;
const LCD_RETURNHOME: u8 = 0x02;
const LCD_ENTRYMODESET: u8 = 0x04;
const LCD_DISPLAYCONTROL: u8 = 0x08;
const LCD_CURSORSHIFT: u8 = 0x10;
const LCD_FUNCTIONSET: u8 = 0x20;
const LCD_SETCGRAMADDR: u8 = 0x40;
const LCD_SETDDRAMADDR: u8 = 0x80;

// flags for display entry mode
const LCD_ENTRYRIGHT: u8 = 0x00;
const LCD_ENTRYLEFT: u8 = 0x02;
const LCD_ENTRYSHIFTINCREMENT: u8 = 0x01;
const LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;

// flags for display on/off control
const LCD_DISPLAYON: u8 = 0x04;
const LCD_DISPLAYOFF: u8 = 0x00;
const LCD_CURSORON: u8 = 0x02;
const LCD_CURSOROFF: u8 = 0x00;
const LCD_BLINKON: u8 = 0x01;
const LCD_BLINKOFF: u8 = 0x00;

// flags for display/cursor shift
const LCD_DISPLAYMOVE: u8 = 0x08;
const LCD_CURSORMOVE: u8 = 0x00;
const LCD_MOVERIGHT: u8 = 0x04;
const LCD_MOVELEFT: u8 = 0x00;

// flags for function set
const LCD_8BITMODE: u8 = 0x10;
const LCD_4BITMODE: u8 = 0x00;
const LCD_2LINE: u8 = 0x08;
const LCD_1LINE: u8 = 0x00;
const LCD_5X10_DOTS: u8 = 0x04;
const LCD_5X8_DOTS: u8 = 0x00;

// flags for backlight control
const LCD_BACKLIGHT: u8 = 0x08;
const LCD_NOBACKLIGHT: u8 = 0x00;

const EN: u8 = 0b00000100; // Enable bit
const RW: u8 = 0b00000010; // Read/Write bit
const RS: u8 = 0b00000001; // Register select bit

pub struct LcdI2c {
    address: u8,
    display_function: u8,
    display_control: u8,
    display_mode: u8,
    num_lines: u8,
    oled: bool,
    columns: u8,
    rows: u8,
    backlight_value: u8,
    i2c: arduino_hal::I2c,
}

impl LcdI2c {
    pub fn new(address: u8, oled: bool, columns: u8, rows: u8, backlight_value: u8) -> Self {
        let dp = arduino_hal::Peripherals::take().unwrap();
        let pins = arduino_hal::pins!(dp);
        let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

        let i2c = arduino_hal::I2c::new(
            dp.TWI,
            pins.a4.into_pull_up_input(),
            pins.a5.into_pull_up_input(),
            50000,
        );

        Self {
            address,
            display_function: 0,
            display_control: 0,
            display_mode: 0,
            num_lines: 2,
            oled: false,
            columns,
            rows,
            backlight_value,
            i2c,
        }
    }

    pub fn begin(&mut self, cols: u8, lines: u8, dotsize: Option<u8>) {
        if lines > 1 {
            self.display_function |= LCD_2LINE;
        }
        self.num_lines = lines;

        let dotsize = match dotsize {
            Some(size) => size,
            None => LCD_5X8_DOTS,
        };

        if dotsize != 0 && lines == 1 {
            self.display_function |= LCD_5X10_DOTS;
        }

        arduino_hal::delay_ms(50);

        self.expander_write(self.backlight_value);
        arduino_hal::delay_ms(1000);

        self.write_four_bits(0x03 << 4);
        arduino_hal::delay_us(4500);

        self.write_four_bits(0x03 << 4);
        arduino_hal::delay_us(4500);

        self.write_four_bits(0x03 << 4);
        arduino_hal::delay_us(4500);

        self.write_four_bits(0x02 << 4);

        self.command(LCD_FUNCTIONSET | self.display_function);

        self.display_control = LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF;
        self.display();

        self.clear();

        self.display_mode = LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT;

        self.command(LCD_ENTRYMODESET | self.display_mode);

        self.home();
    }

    pub fn clear(&mut self) {
        self.command(LCD_CLEARDISPLAY);
        arduino_hal::delay_us(2000);
        if self.oled {
            self.set_cursor(0, 0);
        }
    }

    pub fn home(&mut self) {
        self.command(LCD_RETURNHOME);
        arduino_hal::delay_us(2000);
    }

    pub fn no_display(&mut self) {
        self.display_control &= !LCD_DISPLAYON;
        self.command(LCD_DISPLAYCONTROL | self.display_control);
    }

    pub fn display(&mut self) {
        self.display_control |= LCD_DISPLAYON;
        self.command(LCD_DISPLAYCONTROL | self.display_control);
    }

    pub fn no_blink(&mut self) {
        self.display_control &= !LCD_BLINKON;
        self.command(LCD_DISPLAYCONTROL | self.display_control);
    }

    pub fn blink(&mut self) {
        self.display_control |= LCD_BLINKON;
        self.command(LCD_DISPLAYCONTROL | self.display_control);
    }

    pub fn no_cursor(&mut self) {
        self.display_control &= !LCD_CURSORON;
        self.command(LCD_DISPLAYCONTROL | self.display_control);
    }

    pub fn cursor(&mut self) {
        self.display_control |= LCD_CURSORON;
        self.command(LCD_DISPLAYCONTROL | self.display_control);
    }

    pub fn scroll_display_left(&mut self) {
        self.command(LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVELEFT);
    }

    pub fn scroll_display_right(&mut self) {
        self.command(LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVERIGHT);
    }

    pub fn left_to_right(&mut self) {
        self.display_mode |= LCD_ENTRYLEFT;
        self.command(LCD_ENTRYMODESET | self.display_mode);
    }

    pub fn right_to_left(&mut self) {
        self.display_mode |= LCD_ENTRYRIGHT;
        self.command(LCD_ENTRYMODESET | self.display_mode);
    }

    pub fn no_backlight(&mut self) {
        self.backlight_value = LCD_NOBACKLIGHT;
        self.expander_write(0);
    }

    pub fn backlight(&mut self) {
        self.backlight_value = LCD_BACKLIGHT;
        self.expander_write(0);
    }

    pub fn autoscroll(&mut self) {
        self.display_mode |= LCD_ENTRYSHIFTINCREMENT;
        self.command(LCD_ENTRYMODESET | self.display_mode);
    }

    pub fn no_autoscroll(&mut self) {
        self.display_mode &= !LCD_ENTRYSHIFTINCREMENT;
        self.command(LCD_ENTRYMODESET | self.display_mode);
    }

    pub fn set_cursor(&mut self, column: u8, row: u8) {
        let row_offsets: [u8; 4] = [0x00, 0x40, 0x14, 0x54];
        let mut row = row;
        if row > self.num_lines {
            row = self.num_lines - 1;
        }
        self.command(LCD_SETDDRAMADDR | (column + row_offsets[row as usize]));
    }

    pub fn command(&mut self, value: u8) {
        self.send(value, 0);
    }

    pub fn init(&mut self) {
        self.init_private();
    }

    pub fn oled_init(&mut self) {
        self.oled = true;
        self.init();
    }

    pub fn print() {}

    pub fn println() {}

    fn init_private(&mut self) {
        self.display_function = LCD_4BITMODE | LCD_1LINE | LCD_5X8_DOTS;
        self.begin(self.columns, self.rows, None);
    }

    fn send(&mut self, value: u8, mode: u8) {
        let highnib: u8 = value & 0xf0;
        let lownib: u8 = (value << 4) & 0xf0;

        self.write_four_bits((highnib) | mode);
        self.write_four_bits((lownib) | mode);
    }

    fn write_four_bits(&mut self, value: u8) {
        self.expander_write(value);
        self.pulse_enable(value);
    }

    fn expander_write(&mut self, data: u8) {
        self.i2c
            .write(self.address, &[(data | self.backlight_value)])
            .unwrap();
    }

    fn pulse_enable(&mut self, data: u8) {
        self.expander_write(data | EN);
        arduino_hal::delay_us(1);

        self.expander_write(data & !EN);
        arduino_hal::delay_us(50);
    }

    fn write(&mut self, value: u8) {
        self.send(value, RS);
    }
}

/*
   # Todo
   - genauer nach schauen wie ich write implmentieren soll
*/
