#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use panic_probe as _;

// I2C
use embassy_rp::i2c::Config as I2cConfig;
use embassy_rp::i2c::{self}; // for convenience, importing as alias

// LCD Driver
use hd44780_driver::HD44780;

use embassy_time::Delay;

const LCD_I2C_ADDRESS: u8 = 0x27;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Initializing the program");

    let sda = p.PIN_16;
    let scl = p.PIN_17;

    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = 100_000; //100kHz

    let i2c = i2c::I2c::new_blocking(p.I2C0, scl, sda, i2c_config);

    // LCD Init
    let mut lcd =
        HD44780::new_i2c(i2c, LCD_I2C_ADDRESS, &mut Delay).expect("failed to initialize lcd");

    // Clear the screen
    lcd.reset(&mut Delay).expect("failed to reset lcd screen");
    lcd.clear(&mut Delay).expect("failed to clear the screen");

    // Write to the top line
    lcd.write_str("Hello, Rust!", &mut Delay)
        .expect("failed to write text to LCD");

    loop {
        Timer::after_secs(1).await;
    }
}
