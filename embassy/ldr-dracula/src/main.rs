#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use panic_probe as _;

// For Interrupt Binding
use embassy_rp::adc::InterruptHandler;
use embassy_rp::bind_interrupts;

// For ADC
use embassy_rp::adc::{Adc, Channel, Config as AdcConfig};

// For LED
use embassy_rp::gpio::{Level, Output, Pull};

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => InterruptHandler;
});

const LDR_THRESHOLD: u16 = 200;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Initializing the program");

    let mut adc = Adc::new(p.ADC, Irqs, AdcConfig::default());

    let mut adc_pin = Channel::new_pin(p.PIN_28, Pull::None);

    let mut led = Output::new(p.PIN_15, Level::Low);

    loop {
        let adc_reading = adc
            .read(&mut adc_pin)
            .await
            .expect("Unable to read the adc value");
        defmt::info!("ADC value: {}", adc_reading);

        if adc_reading < LDR_THRESHOLD {
            led.set_high();
        } else {
            led.set_low();
        }

        Timer::after_secs(1).await;
    }
}
