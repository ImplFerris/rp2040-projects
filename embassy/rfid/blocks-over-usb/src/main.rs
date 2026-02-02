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

// to prepare a buffer for formatted output
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

fn print_hex(data: &[u8]) {
    let mut buff: String<64> = String::new();
    for &d in data.iter() {
        write!(buff, "{:02x} ", d).expect("failed to write byte into buffer");
    }
    log::info!("{}", buff);
}

fn read_sector<E, COMM>(
    uid: &mfrc522::Uid,
    sector: u8,
    rfid: &mut Mfrc522<COMM, mfrc522::Initialized>,
) -> Result<(), &'static str>
where
    COMM: mfrc522::comm::Interface<Error = E>,
{
    const AUTH_KEY: [u8; 6] = [0xFF; 6];

    let block_offset = sector * 4;
    rfid.mf_authenticate(uid, block_offset, &AUTH_KEY)
        .map_err(|_| "Auth failed")?;

    for abs_block in block_offset..block_offset + 4 {
        let data = rfid.mf_read(abs_block).map_err(|_| "Read failed")?;
        print_hex(&data);
    }
    Ok(())
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
    let mut rfid = Mfrc522::new(itf)
        .init()
        .expect("failed to initialize the RFID reader");

    loop {
        if let Ok(atqa) = rfid.reqa() {
            if let Ok(uid) = rfid.select(&atqa) {
                if let Err(e) = read_sector(&uid, 0, &mut rfid) {
                    log::error!("Error reading sector: {:?}", e);
                }
                let _ = rfid.hlta();
                let _ = rfid.stop_crypto1();
                Timer::after_millis(100).await;
            }
        }

        Timer::after_millis(100).await;
    }
}
