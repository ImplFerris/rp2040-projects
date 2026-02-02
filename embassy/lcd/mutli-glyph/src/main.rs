#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Delay, Timer};

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use panic_probe as _;

// Interrupt Binding
use embassy_rp::peripherals::I2C0;
use embassy_rp::{bind_interrupts, i2c};

// I2C
use embassy_rp::i2c::{Config as I2cConfig, I2c};

// LCD Driver
use liquid_crystal::I2C;
use liquid_crystal::LiquidCrystal;
use liquid_crystal::prelude::*;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

const SYMBOL1: [u8; 8] = [
    0b00110, 0b01000, 0b01110, 0b01000, 0b00100, 0b00011, 0b00100, 0b01000,
];

const SYMBOL2: [u8; 8] = [
    0b00000, 0b00000, 0b00000, 0b10001, 0b10001, 0b11111, 0b00000, 0b00000,
];

const SYMBOL3: [u8; 8] = [
    0b01100, 0b00010, 0b01110, 0b00010, 0b00100, 0b11000, 0b00100, 0b00010,
];

const SYMBOL4: [u8; 8] = [
    0b01000, 0b01000, 0b00100, 0b00011, 0b00001, 0b00010, 0b00101, 0b01000,
];

const SYMBOL5: [u8; 8] = [
    0b00000, 0b00000, 0b00000, 0b11111, 0b01010, 0b10001, 0b00000, 0b00000,
];

const SYMBOL6: [u8; 8] = [
    0b00010, 0b00010, 0b00100, 0b11000, 0b10000, 0b01000, 0b10100, 0b00010,
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Initializing the program");

    let sda = p.PIN_16;
    let scl = p.PIN_17;

    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = 100_000; // 100kHz

    let i2c_bus = I2c::new_async(p.I2C0, scl, sda, Irqs, i2c_config);

    // LCD Init
    let mut i2c_interface = I2C::new(i2c_bus, 0x27);
    let mut lcd = LiquidCrystal::new(&mut i2c_interface, Bus4Bits, LCD16X2);
    lcd.begin(&mut Delay);

    lcd.custom_char(&mut Delay, &SYMBOL1, 0);
    lcd.custom_char(&mut Delay, &SYMBOL2, 1);
    lcd.custom_char(&mut Delay, &SYMBOL3, 2);
    lcd.custom_char(&mut Delay, &SYMBOL4, 3);
    lcd.custom_char(&mut Delay, &SYMBOL5, 4);
    lcd.custom_char(&mut Delay, &SYMBOL6, 5);

    lcd.set_cursor(&mut Delay, 0, 4)
        .write(&mut Delay, CustomChar(0))
        .write(&mut Delay, CustomChar(1))
        .write(&mut Delay, CustomChar(2));

    lcd.set_cursor(&mut Delay, 1, 4)
        .write(&mut Delay, CustomChar(3))
        .write(&mut Delay, CustomChar(4))
        .write(&mut Delay, CustomChar(5));

    loop {
        Timer::after_secs(1).await;
    }
}
