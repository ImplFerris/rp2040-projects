#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

// defmt Logging
use defmt::{error, info};
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

fn print_hex(data: &[u8]) {
    let mut buff: String<64> = String::new();
    for &d in data.iter() {
        write!(buff, "{:02x} ", d).expect("failed to write byte into buffer");
    }
    defmt::println!("{}", buff);
}

fn read_sector<E, COMM>(
    uid: &mfrc522::Uid,
    sector: u8,
    key: &[u8; 6],
    rfid: &mut Mfrc522<COMM, mfrc522::Initialized>,
) -> Result<(), &'static str>
where
    COMM: mfrc522::comm::Interface<Error = E>,
{
    let block_offset = sector * 4;
    rfid.mf_authenticate(uid, block_offset, key)
        .map_err(|_| "Auth failed")?;

    for abs_block in block_offset..block_offset + 4 {
        let data = rfid.mf_read(abs_block).map_err(|_| "Read failed")?;
        print_hex(&data);
    }
    Ok(())
}

fn write_block<E, COMM>(
    uid: &mfrc522::Uid,
    sector: u8,
    rel_block: u8,
    data: [u8; 16],
    key: &[u8; 6],
    rfid: &mut Mfrc522<COMM, mfrc522::Initialized>,
) -> Result<(), &'static str>
where
    COMM: mfrc522::comm::Interface<Error = E>,
{
    let block_offset = sector * 4;
    let abs_block = block_offset + rel_block;

    rfid.mf_authenticate(uid, block_offset, key)
        .map_err(|_| "Auth failed")?;

    rfid.mf_write(abs_block, data).map_err(|_| "Write failed")?;

    Ok(())
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

    let target_sector = 1;
    let rel_block = 3;
    const DATA: [u8; 16] = [
        0x52, 0x75, 0x73, 0x74, 0x65, 0x64, // Key A: "Rusted"
        0xFF, 0x07, 0x80, 0x69, // Access bits and trailer byte
        0x46, 0x65, 0x72, 0x72, 0x69, 0x73, // Key B: "Ferris"
    ];
    let current_key = &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    let new_key: &[u8; 6] = &DATA[..6].try_into().expect("have enough data");

    loop {
        if let Ok(atqa) = rfid.reqa() {
            if let Ok(uid) = rfid.select(&atqa) {
                defmt::println!("\r\n----Before Write----\r\n");
                if let Err(e) = read_sector(&uid, target_sector, current_key, &mut rfid) {
                    error!("Error reading sector: {:?}", e);
                }
                Timer::after_millis(200).await;

                if let Err(e) =
                    write_block(&uid, target_sector, rel_block, DATA, current_key, &mut rfid)
                {
                    error!("Error writing block: {:?}", e);
                }
                Timer::after_millis(200).await;

                defmt::println!("\r\n----After Write----\r\n");
                if let Err(e) = read_sector(&uid, target_sector, new_key, &mut rfid) {
                    error!("Error reading sector: {:?}", e);
                }

                let _ = rfid.hlta();
                let _ = rfid.stop_crypto1();
                Timer::after_millis(500).await;
            }
        }

        Timer::after_millis(200).await;
    }
}
