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

    let out_pin = common.make_pio_pin(p.PIN_15);

    let prg = pio_asm!(
        "
        set pindirs, 1
        loop:
        set pins, 1 [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        set pins, 0 [30]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        nop [31]
        jmp loop
        "
    );

    let mut cfg = Config::default();
    cfg.use_program(&common.load_program(&prg.program), &[]);
    cfg.set_set_pins(&[&out_pin]);
    cfg.clock_divider = 65535u16.into();
    sm0.set_config(&cfg);

    sm0.set_enable(true);

    let mut counter: u8 = 0;
    info!("Counter that running on main cpu - not related to the  PIO");
    loop {
        info!("Count: {}", counter);

        counter = counter.wrapping_add(1);

        if counter == 0 {
            info!("Wrapped the counter");
        }

        Timer::after_millis(100).await;
    }
}
