#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use panic_probe as _;

use embassy_rp::pwm::{Config as PwmConfig, Pwm, SetDutyCycle};

const fn get_top(freq: f64, div_int: u8) -> u16 {
    assert!(div_int != 0, "Divider must not be 0");

    let result = 125_000_000. / (freq * div_int as f64);

    assert!(result >= 1.0, "Frequency too high");
    assert!(
        result <= 65535.0,
        "Frequency too low: TOP exceeds 65534 max"
    );

    result as u16 - 1
}

const PWM_DIV_INT: u8 = 64;
const PWM_TOP: u16 = get_top(440., PWM_DIV_INT);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Initializing the program");

    let mut pwm_config = PwmConfig::default();
    pwm_config.top = PWM_TOP;
    pwm_config.divider = PWM_DIV_INT.into();

    let mut buzzer = Pwm::new_output_b(p.PWM_SLICE7, p.PIN_15, pwm_config);

    loop {
        buzzer
            .set_duty_cycle_percent(50)
            .expect("50 is valid duty percentage");
        Timer::after_millis(1000).await;

        buzzer
            .set_duty_cycle_percent(0)
            .expect("0 is valid duty percentage");
        Timer::after_millis(1000).await;
    }
}
