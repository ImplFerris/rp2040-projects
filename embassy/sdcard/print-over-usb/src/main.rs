#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

use panic_halt as _;

// For SPI
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;

// For CS Pin
use embassy_rp::gpio::{Level, Output};

// For SdCard
use embedded_sdmmc::{SdCard, TimeSource, Timestamp, VolumeIdx, VolumeManager};

// logger
use log::info;

// For USB
use embassy_rp::{peripherals::USB, usb};

embassy_rp::bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(usb: embassy_rp::Peri<'static, embassy_rp::peripherals::USB>) {
    let driver = embassy_rp::usb::Driver::new(usb, Irqs);

    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

/// Code from https://github.com/rp-rs/rp-hal-boards/blob/main/boards/rp-pico/examples/pico_spi_sd_card.rs
/// A dummy timesource, which is mostly important for creating files.
#[derive(Default)]
pub struct DummyTimesource();

impl TimeSource for DummyTimesource {
    // In theory you could use the RTC of the rp2040 here, if you had
    // any external time synchronizing device.
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    spawner.must_spawn(logger_task(p.USB));

    let miso = p.PIN_4;
    let cs_pin = Output::new(p.PIN_5, Level::High);
    let clk = p.PIN_6;
    let mosi = p.PIN_7;

    let mut config = spi::Config::default();
    config.frequency = 400_000;

    let spi_bus = Spi::new_blocking(p.SPI0, clk, mosi, miso, config);

    let spi_device =
        ExclusiveDevice::new(spi_bus, cs_pin, Delay).expect("Failed to get exclusive device");

    let sdcard = SdCard::new(spi_device, Delay);

    info!("Init SD card controller and retrieve card size...");
    let sd_size = sdcard.num_bytes().expect("failed to get sdcard size");
    info!("card size is {} bytes", sd_size);

    let volume_mgr = VolumeManager::new(sdcard, DummyTimesource::default());
    let volume0 = volume_mgr
        .open_volume(VolumeIdx(0))
        .expect("failed to open volume");

    let root_dir = volume0.open_root_dir().expect("failed to open root dir");

    let my_file = root_dir
        .open_file_in_dir("RUST.TXT", embedded_sdmmc::Mode::ReadOnly)
        .expect("failed to open RUST.TXT file");

    while !my_file.is_eof() {
        let mut buffer = [0u8; 32];

        if let Ok(n) = my_file.read(&mut buffer) {
            if let Ok(s) = core::str::from_utf8(&buffer[..n]) {
                info!("{}", s);
            } else {
                info!("{:02x?}", &buffer[..n]);
            }
        }
    }

    loop {
        Timer::after_secs(1).await;
    }
}
