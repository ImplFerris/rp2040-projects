#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

use panic_halt as _;

// For USB
use embassy_rp::{peripherals::USB, usb};

// For SPI
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;

// For CS Pin
use embassy_rp::gpio::{Level, Output};

// Driver for the MFRC522
use mfrc522::{Mfrc522, comm::blocking::spi::SpiInterface};

// to prepare buffer with data before writing into USB serial
use core::fmt::Write;
use heapless::String;

embassy_rp::bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(usb: embassy_rp::Peri<'static, embassy_rp::peripherals::USB>) {
    let driver = embassy_rp::usb::Driver::new(usb, Irqs);

    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

fn print_hex_to_serial(data: &[u8]) {
    let mut buff: String<64> = String::new();
    for &d in data.iter() {
        write!(buff, "{:02x} ", d).expect("failed to write byte into buffer");
    }
    log::info!("UID: {}", buff);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    spawner.must_spawn(logger_task(p.USB));
    Timer::after_secs(3).await;

    let miso = p.PIN_0;
    let cs_pin = Output::new(p.PIN_1, Level::High);
    let clk = p.PIN_2;
    let mosi = p.PIN_3;

    let mut config = spi::Config::default();
    config.frequency = 1000_000;

    let spi_bus = Spi::new_blocking(p.SPI0, clk, mosi, miso, config);
    let spi = ExclusiveDevice::new(spi_bus, cs_pin, Delay).expect("Failed to get exclusive device");

    let itf = SpiInterface::new(spi);

    log::info!("Initializing MFRC522...");
    let mut rfid = match Mfrc522::new(itf).init() {
        Ok(rfid) => {
            log::info!("MFRC522 initialized successfully");
            rfid
        }
        Err(e) => {
            log::error!("Failed to initialize MFRC522: {:?}", e);
            loop {
                Timer::after_secs(1).await;
            }
        }
    };

    log::info!("Waiting for RFID");
    loop {
        if let Ok(atqa) = rfid.reqa() {
            if let Ok(uid) = rfid.select(&atqa) {
                print_hex_to_serial(uid.as_bytes());
                Timer::after_millis(500).await;
            }
        }
        Timer::after_millis(100).await;
    }
}
