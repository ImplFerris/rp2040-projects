#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use panic_probe as _;

use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Config, InterruptHandler, Pio};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Initializing the program");

    let pio = p.PIO0;
    let Pio {
        mut common,
        mut sm0,
        ..
    } = Pio::new(pio, Irqs);

    let prg = pio_asm!(
        "
    set pindirs, 1
    .wrap_target
        out pins,1 [31]
    .wrap
    "
    );

    let out_pin = common.make_pio_pin(p.PIN_15);

    let mut cfg = Config::default();
    cfg.use_program(&common.load_program(&prg.program), &[]);
    cfg.set_set_pins(&[&out_pin]);
    cfg.clock_divider = 65535u16.into();

    cfg.set_out_pins(&[&out_pin]);
    cfg.shift_out.auto_fill = true;

    sm0.set_config(&cfg);

    sm0.set_enable(true);

    let patterns = [
        0b10000000_10000000_10000000_10000000,
        0b11000000_11000000_11000000_11000000,
        0b11110000_11110000_11110000_11110000,
        0b11111100_11111100_11111100_11111100,
        0b00000000_00000000_00000000_00000000,
    ];

    loop {
        for pattern in &patterns {
            sm0.tx().wait_push(*pattern).await;
            info!("Pushed pattern to FIFO {:032b}", pattern);
            Timer::after_millis(100).await;
        }

        Timer::after_secs(3).await;
    }
}
