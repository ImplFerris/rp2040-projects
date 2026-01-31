#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use embassy_rp::gpio::{Level, Output};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Initializing the program");

    let mut led = Output::new(p.PIN_13, Level::Low);

    loop {
        led.set_high(); // Turn on the LED
        Timer::after_millis(500).await;

        led.set_low(); // Turn off the LED
        Timer::after_millis(500).await;
    }
}
