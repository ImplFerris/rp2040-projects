#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use panic_probe as _;

// Text formatting without heap allocation
use core::fmt::Write;
use heapless::String;

// For OLED display
use ssd1306::{I2CDisplayInterface, Ssd1306Async, prelude::*};

// For ADC
use embassy_rp::adc::{Adc, Channel, Config as AdcConfig};
use embassy_rp::gpio::Pull;

// Interrupt Binding
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::I2C0;
use embassy_rp::{adc, i2c};

// I2C
use embassy_rp::i2c::{Config as I2cConfig, I2c};

// Embedded Graphics
use embedded_graphics::{
    mono_font::{MonoTextStyle, iso_8859_13::FONT_7X13_BOLD},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});

const ADC_LEVELS: f64 = 4096.0;

const B_VALUE: f64 = 3950.0;
const REF_RES: f64 = 10_000.0; // Reference resistance in ohms (10kΩ)
const REF_TEMP: f64 = 25.0; // Reference temperature 25°C

// We have already covered about this formula in ADC chapter
fn adc_to_resistance(adc_value: u16, r2_res: f64) -> f64 {
    let adc = adc_value as f64;
    ((ADC_LEVELS / adc) - 1.0) * r2_res
}

// B Equation to convert resistance to temperature
fn calculate_temperature(current_res: f64, ref_res: f64, ref_temp: f64, b_val: f64) -> f64 {
    let ln_value = libm::log(current_res / ref_res); // Use libm for `no_std`
    let inv_t = (1.0 / ref_temp) + ((1.0 / b_val) * ln_value);
    1.0 / inv_t
}

fn kelvin_to_celsius(kelvin: f64) -> f64 {
    kelvin - 273.15
}

fn celsius_to_kelvin(celsius: f64) -> f64 {
    celsius + 273.15
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Initializing the program");

    // Display Setup
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

    let text_style = MonoTextStyle::new(&FONT_7X13_BOLD, BinaryColor::On);

    // ADC Setup for thermistor
    let mut adc_pin = Channel::new_pin(p.PIN_28, Pull::None);
    let mut adc = Adc::new(p.ADC, Irqs, AdcConfig::default());

    let mut buff: String<64> = String::new();

    let ref_temp = celsius_to_kelvin(REF_TEMP);

    loop {
        buff.clear();
        display
            .clear(BinaryColor::Off)
            .expect("failed to clear the display");

        let adc_value = adc
            .read(&mut adc_pin)
            .await
            .expect("failed to read adc value");

        let current_res = adc_to_resistance(adc_value, REF_RES);

        let temperature_kelvin = calculate_temperature(current_res, REF_RES, ref_temp, B_VALUE);
        let temperature_celsius = kelvin_to_celsius(temperature_kelvin);

        writeln!(buff, "Temp: {:.2} °C", temperature_celsius)
            .expect("failed to format temperature");

        writeln!(buff, "ADC: {}", adc_value).expect("failed to format ADC value");

        writeln!(buff, "R: {:.2}", current_res).expect("failed to format Resistance");

        Text::new(&buff, Point::new(5, 20), text_style)
            .draw(&mut display)
            .expect("Failed to write the text");

        display.flush().await.expect("failed to send to display");

        Timer::after_secs(2).await;
    }
}
