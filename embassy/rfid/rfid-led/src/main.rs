#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use panic_probe as _;

// For SPI
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;

// For CS Pin
use embassy_rp::gpio::{Level, Output};

// Driver for the MFRC522
use mfrc522::{Mfrc522, comm::blocking::spi::SpiInterface};

// Replace the UID Bytes with your tag UID
const TAG_UID: [u8; 4] = [0x13, 0x37, 0x73, 0x31];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Initializing the program");

    let miso = p.PIN_0;
    let cs_pin = Output::new(p.PIN_1, Level::High);
    let clk = p.PIN_2;
    let mosi = p.PIN_3;

    let mut config = spi::Config::default();
    config.frequency = 1000_000;

    let spi_bus = Spi::new_blocking(p.SPI0, clk, mosi, miso, config);

    let spi = ExclusiveDevice::new(spi_bus, cs_pin, Delay).expect("Failed to get exclusive device");

    let itf = SpiInterface::new(spi);
    let mut rfid = Mfrc522::new(itf)
        .init()
        .expect("failed to initialize the RFID reader");

    // On board LED only works for Pico not for Pico W
    // let mut led = Output::new(p.PIN_25, Level::Low);

    // Using External LED, connected to GPIO 15
    let mut led = Output::new(p.PIN_15, Level::Low);

    loop {
        led.set_low();

        if let Ok(atqa) = rfid.reqa() {
            if let Ok(uid) = rfid.select(&atqa) {
                if *uid.as_bytes() == TAG_UID {
                    led.set_high();
                    Timer::after_millis(500).await;
                }
            }
        }
        Timer::after_millis(100).await;
    }
}
