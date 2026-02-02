#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use panic_probe as _;

// For ADC
use embassy_rp::adc::{Adc, Channel, Config as AdcConfig};
use embassy_rp::gpio::{Input, Pull};

// Interrupt Binding
use embassy_rp::adc;
use embassy_rp::bind_interrupts;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => adc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Initializing the program");

    // ADC Setup
    let mut adc = Adc::new(p.ADC, Irqs, AdcConfig::default());

    let mut vrx_pin = Channel::new_pin(p.PIN_27, Pull::None);
    let mut vry_pin = Channel::new_pin(p.PIN_26, Pull::None);
    let button = Input::new(p.PIN_15, Pull::Up);

    let mut prev_vrx: u16 = 0;
    let mut prev_vry: u16 = 0;
    let mut print_vals = true;
    let mut prev_btn_state = false;

    loop {
        let Ok(vry) = adc.read(&mut vry_pin).await else {
            continue;
        };
        let Ok(vrx) = adc.read(&mut vrx_pin).await else {
            continue;
        };

        if vrx.abs_diff(prev_vrx) > 100 {
            prev_vrx = vrx;
            print_vals = true;
        }

        if vry.abs_diff(prev_vry) > 100 {
            prev_vry = vry;
            print_vals = true;
        }

        if print_vals {
            print_vals = false;

            info!("X: {} Y: {}", vrx, vry);
        }

        let btn_state = button.is_low();
        if btn_state && !prev_btn_state {
            info!("Button Pressed");

            print_vals = true;
        }
        prev_btn_state = btn_state;

        Timer::after_millis(100).await;
    }
}
