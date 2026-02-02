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

// to prepare buffer with data before logging
use core::fmt::Write;
use heapless::String;

fn read_sector<E, COMM>(
    uid: &mfrc522::Uid,
    sector: u8,
    rfid: &mut Mfrc522<COMM, mfrc522::Initialized>,
) -> Result<(), &'static str>
where
    COMM: mfrc522::comm::Interface<Error = E>,
{
    const AUTH_KEY: [u8; 6] = [0xFF; 6];

    let mut buff: String<64> = String::new();

    let block_offset = sector * 4;
    rfid.mf_authenticate(uid, block_offset, &AUTH_KEY)
        .map_err(|_| "Auth failed")?;

    for abs_block in block_offset..block_offset + 4 {
        let rel_block = abs_block - block_offset;
        let data = rfid.mf_read(abs_block).map_err(|_| "Read failed")?;

        // Printing the block data
        for &d in data.iter() {
            write!(buff, "{:02x} ", d).expect("failed to write byte into buffer");
        }

        // Printing block type
        let block_type = get_block_type(sector, rel_block);

        defmt::println!(
            "BLOCK {} (REL: {}) | {} | {}",
            abs_block,
            rel_block,
            buff,
            block_type
        );

        buff.clear();
    }
    defmt::println!("");
    Ok(())
}

fn dump_memory<E, COMM>(
    uid: &mfrc522::Uid,
    rfid: &mut Mfrc522<COMM, mfrc522::Initialized>,
) -> Result<(), &'static str>
where
    COMM: mfrc522::comm::Interface<Error = E>,
{
    let mut buff: String<64> = String::new();
    for sector in 0..16 {
        // Printing the Sector number
        write!(buff, "-----------SECTOR {}-----------", sector)
            .expect("failed to write into heapless buff");
        defmt::println!("{}", buff);
        buff.clear();

        read_sector(uid, sector, rfid)?;
    }
    Ok(())
}

fn get_block_type(sector: u8, rel_block: u8) -> &'static str {
    match rel_block {
        0 if sector == 0 => "MFD",
        3 => "TRAILER",
        _ => "DATA",
    }
}

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

    loop {
        if let Ok(atqa) = rfid.reqa() {
            if let Ok(uid) = rfid.select(&atqa) {
                if let Err(e) = dump_memory(&uid, &mut rfid) {
                    defmt::error!("Error reading sector: {:?}", e);
                }
                let _ = rfid.hlta();
                let _ = rfid.stop_crypto1();
                Timer::after_millis(500).await;
            }
        }

        Timer::after_millis(200).await;
    }
}
