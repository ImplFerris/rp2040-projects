#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use panic_probe as _;

// For PWM
use embassy_rp::pwm::{Config as PwmConfig, Pwm, SetDutyCycle};

const PWM_DIV_INT: u8 = 64;
const PWM_TOP: u16 = 39_061;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Initializing the program");

    let mut servo_config: PwmConfig = Default::default();
    servo_config.top = PWM_TOP;
    servo_config.divider = PWM_DIV_INT.into();

    let mut servo = Pwm::new_output_b(p.PWM_SLICE7, p.PIN_15, servo_config);

    loop {
        // Move servo to 0° position (2.5% duty cycle = 25/1000)
        servo
            .set_duty_cycle_fraction(25, 1000)
            .expect("invalid min duty cycle");

        Timer::after_millis(1000).await;

        // 90° position (7.5% duty cycle)
        servo
            .set_duty_cycle_fraction(75, 1000)
            .expect("invalid half duty cycle");

        Timer::after_millis(1000).await;

        // 180° position (12% duty cycle)
        servo
            .set_duty_cycle_fraction(120, 1000)
            .expect("invalid max duty cycle");

        Timer::after_millis(1000).await;
    }
}
