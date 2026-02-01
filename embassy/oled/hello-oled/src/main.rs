#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use panic_probe as _;

// Interrupt Binding
use embassy_rp::peripherals::I2C0;
use embassy_rp::{bind_interrupts, i2c};

// I2C
use embassy_rp::i2c::{Config as I2cConfig, I2c};

// OLED
use ssd1306::{I2CDisplayInterface, Ssd1306Async, prelude::*};

// Embedded Graphics
use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    pixelcolor::BinaryColor,
    prelude::Point,
    prelude::*,
    text::{Baseline, Text},
};

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Initializing the program");

    let sda = p.PIN_16;
    let scl = p.PIN_17;

    let mut i2c_config = I2cConfig::default();
    i2c_config.frequency = 400_000; //400kHz

    let i2c_bus = I2c::new_async(p.I2C0, scl, sda, Irqs, i2c_config);

    let i2c_interface = I2CDisplayInterface::new(i2c_bus);

    let mut display = Ssd1306Async::new(i2c_interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display
        .init()
        .await
        .expect("failed to initialize the display");

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    defmt::info!("sending text to display");
    Text::with_baseline("Hello, Rust!", Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .expect("failed to draw text to display");

    display
        .flush()
        .await
        .expect("failed to flush data to display");

    loop {
        Timer::after_secs(1).await;
    }
}
