use arduino_hal::prelude::*;

const MPU6050_SMPLRT_DIV_REGISTER: u8 = 0x19;
const MPU6050_CONFIG_REGISTER: u8 = 0x1a;
const MPU6050_GYRO_CONFIG_REGISTER: u8 = 0x1b;
const MPU6050_ACCEL_CONFIG_REGISTER: u8 = 0x1c;
const MPU6050_PWR_MGMT_1_REGISTER: u8 = 0x6b;

const MPU6050_GYRO_OUT_REGISTER: u8 = 0x43;
const MPU6050_ACCEL_OUT_REGISTER: u8 = 0x3B;

const RAD_2_DEG: f32 = 57.29578;
const CALIB_OFFSET_NB_MES: u16 = 500;
const TEMP_LSB_2_DEGREE: f32 = 340.0;
const TEMP_LSB_OFFSET: f32 = 12412.0;

const DEFAULT_GYRO_COEFF: f32 = 0.98;

fn wrap(angle: f32, limit: f32) -> f32 {
    let mut angle = angle;
    while angle > limit {
        angle -= 2.0 * limit;
    }

    while angle < -limit {
        angle += 2.0 * limit;
    }

    angle
}

pub struct Mpu6050 {
    address: u8,
    upside_down_mounting: bool,
    gyro_lsb_to_degsec: f32,
    acc_lsb_to_g: f32,
    gyro_offset: (f32, f32, f32),
    acc_offset: (f32, f32, f32),
    temp: f32,
    acc: (f32, f32, f32),
    gyro: (f32, f32, f32),
    angle_acc: (f32, f32),
    angle: (f32, f32, f32),
    pre_interval: u64,
    filter_gyro_coef: f32,
    i2c: arduino_hal::I2c,
}

impl Default for Mpu6050 {
    fn default() -> Self {
        let dp = arduino_hal::Peripherals::take().unwrap();
        let pins = arduino_hal::pins!(dp);
        let _serial = arduino_hal::default_serial!(dp, pins, 57600);

        let i2c = arduino_hal::I2c::new(
            dp.TWI,
            pins.a4.into_pull_up_input(),
            pins.a5.into_pull_up_input(),
            50000,
        );

        Self {
            address: 0x68,
            upside_down_mounting: false,
            gyro_lsb_to_degsec: Default::default(),
            acc_lsb_to_g: Default::default(),
            gyro_offset: Default::default(),
            acc_offset: Default::default(),
            temp: Default::default(),
            acc: Default::default(),
            gyro: Default::default(),
            angle_acc: Default::default(),
            angle: Default::default(),
            pre_interval: Default::default(),
            filter_gyro_coef: DEFAULT_GYRO_COEFF,
            i2c,
        }
    }
}

impl Mpu6050 {
    pub fn new(address: u8) -> Self {
        Self {
            address,
            ..Default::default()
        }
    }

    pub fn set_gyro_offset(&mut self, gyro_offset: (f32, f32, f32)) {
        self.gyro_offset = gyro_offset;
    }

    pub fn set_acc_offset(&mut self, acc_offset: (f32, f32, f32)) {
        self.acc_offset = acc_offset;
    }

    pub fn set_filter_gyro_coef(&mut self, filter_gyro_coef: f32) {
        if filter_gyro_coef < 0.0 || filter_gyro_coef > 1.0 {
            self.filter_gyro_coef = DEFAULT_GYRO_COEFF;
        } else {
            self.filter_gyro_coef = filter_gyro_coef;
        }
    }

    pub fn set_filter_acc_coef(&mut self, filter_acc_coef: f32) {
        self.set_filter_gyro_coef(1.0 - filter_acc_coef);
    }

    pub fn set_upside_down_mounting(&mut self, upside_down_mounting: bool) {
        self.upside_down_mounting = upside_down_mounting;
    }

    pub fn begin(&mut self, gyro_config_num: u8, acc_config_num: u8) {
        let _status: u8 = 0x0;
        self.write_data(MPU6050_PWR_MGMT_1_REGISTER, 0x01);
        self.write_data(MPU6050_SMPLRT_DIV_REGISTER, 0x00);
        self.write_data(MPU6050_CONFIG_REGISTER, 0x00);

        self.set_gyro_config(gyro_config_num);
        self.set_acc_config(acc_config_num);
        self.update();

        self.angle.0 = self.angle_acc.0;
        self.angle.1 = self.angle_acc.1;
        self.pre_interval = crate::millis::millis() as u64;
    }

    pub fn write_data(&mut self, register: u8, data: u8) {
        let sending: [u8; 2] = [register, data];
        self.i2c.write(self.address, &sending).unwrap();
    }

    pub fn read_data(&mut self, register: u8) -> u8 {
        let mut buf: [u8; 1] = [0; 1];
        self.i2c.read(register, &mut buf).unwrap();
        buf[0]
    }

    pub fn set_gyro_config(&mut self, config_num: u8) {
        let gyro_lsb_to_degsec: f32 = match config_num {
            0 => {
                self.write_data(MPU6050_GYRO_CONFIG_REGISTER, 0x00);
                131.0
            }
            1 => {
                self.write_data(MPU6050_GYRO_CONFIG_REGISTER, 0x08);
                65.5
            }
            2 => {
                self.write_data(MPU6050_GYRO_CONFIG_REGISTER, 0x10);
                32.8
            }
            3 => {
                self.write_data(MPU6050_GYRO_CONFIG_REGISTER, 0x18);
                16.4
            }
            _ => {
                self.write_data(MPU6050_GYRO_CONFIG_REGISTER, 0x00);
                131.0
            }
        };

        self.gyro_lsb_to_degsec = gyro_lsb_to_degsec;
    }

    pub fn set_acc_config(&mut self, config_num: u8) {
        let acc_lsb_to_g: f32 = match config_num {
            0 => {
                self.write_data(MPU6050_ACCEL_CONFIG_REGISTER, 0x00);
                16384.0
            }
            1 => {
                self.write_data(MPU6050_ACCEL_CONFIG_REGISTER, 0x08);
                8192.0
            }
            2 => {
                self.write_data(MPU6050_ACCEL_CONFIG_REGISTER, 0x10);
                4096.0
            }
            3 => {
                self.write_data(MPU6050_ACCEL_CONFIG_REGISTER, 0x18);
                2048.0
            }
            _ => {
                self.write_data(MPU6050_ACCEL_CONFIG_REGISTER, 0x00);
                16384.0
            }
        };

        self.acc_lsb_to_g = acc_lsb_to_g;
    }

    pub fn calc_offsets(&mut self, gyro: bool, acc: bool) {
        if gyro {
            self.set_gyro_offset((0.0, 0.0, 0.0));
        }

        if acc {
            self.set_acc_offset((0.0, 0.0, 0.0));
        }

        let mut go: [f32; 3] = [0.0, 0.0, 0.0];
        let mut ao: [f32; 3] = [0.0, 0.0, 0.0];

        for _ in 0..CALIB_OFFSET_NB_MES {
            self.fetch_data();
            go[0] += self.gyro.0;
            go[1] += self.gyro.1;
            go[2] += self.gyro.2;
            ao[0] += self.acc.0;
            ao[1] += self.acc.1;
            ao[2] += self.acc.2;
            arduino_hal::delay_ms(1);
        }

        if gyro {
            self.set_gyro_offset(go.into());
        }

        if acc {
            self.set_acc_offset(ao.into());
        }
    }

    pub fn fetch_data(&mut self) {
        let _ = self.i2c.write(self.address, &[MPU6050_ACCEL_OUT_REGISTER]);

        let mut buf: [u8; 14] = [0; 14];

        let _ = self.i2c.read(self.address, &mut buf);

        // convert buffer to u16
        let mut raw_data: [u16; 7] = [0; 7];

        raw_data[0] = ((buf[0] as u16) << 8) + (buf[1] as u16);
        raw_data[1] = ((buf[2] as u16) << 8) + (buf[3] as u16);
        raw_data[2] = ((buf[4] as u16) << 8) + (buf[5] as u16);
        raw_data[3] = ((buf[6] as u16) << 8) + (buf[7] as u16);
        raw_data[4] = ((buf[8] as u16) << 8) + (buf[9] as u16);
        raw_data[5] = ((buf[10] as u16) << 8) + (buf[11] as u16);
        raw_data[6] = ((buf[12] as u16) << 8) + (buf[13] as u16);

        let mut acc: [f32; 3] = [0.0; 3];

        acc[0] = (raw_data[0] as f32) / self.acc_lsb_to_g - self.acc_offset.0;
        acc[1] = (raw_data[1] as f32) / self.acc_lsb_to_g - self.acc_offset.1;
        acc[2] = ((!self.upside_down_mounting as u8) - (self.upside_down_mounting as u8)) as f32
            * (raw_data[2] as f32)
            / self.acc_lsb_to_g
            - self.acc_offset.2;
    }

    pub fn update(&self) {}
}
