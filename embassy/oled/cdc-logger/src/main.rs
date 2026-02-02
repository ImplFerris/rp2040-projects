#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

use panic_halt as _;

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

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    spawner.must_spawn(logger_task(p.USB));

    let mut i: u8 = 0;
    loop {
        i = i.wrapping_add(1);
        log::info!("USB says: {}", i);

        Timer::after_secs(1).await;
    }
}
