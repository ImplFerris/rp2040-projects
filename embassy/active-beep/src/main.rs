#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use panic_probe as _;

use embassy_rp::gpio::{Level, Output};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Initializing the program");

    let mut buzzer = Output::new(p.PIN_15, Level::Low);

    loop {
        buzzer.set_high();
        Timer::after_millis(500).await;

        buzzer.set_low();
        Timer::after_millis(500).await;
    }
}
