#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use embassy_rp::pwm::{Pwm, SetDutyCycle};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // For external LED on GPIO 15
    let mut pwm = Pwm::new_output_b(p.PWM_SLICE7, p.PIN_15, Default::default());

    // For onboard LED
    // If you are using Pico W, follow the onboard LED steps described in the quick start section.
    // let mut pwm = Pwm::new_output_a(p.PWM_SLICE0, p.PIN_15, Default::default());

    info!("Initializing the program");

    loop {
        for i in 0..=100 {
            Timer::after_millis(8).await;
            let _ = pwm.set_duty_cycle_percent(i);
        }

        for i in (0..=100).rev() {
            Timer::after_millis(8).await;
            let _ = pwm.set_duty_cycle_percent(i);
        }

        Timer::after_millis(500).await;
    }
}
