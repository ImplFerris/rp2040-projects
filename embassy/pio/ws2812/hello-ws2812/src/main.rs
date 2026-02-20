#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::clocks::clk_sys_freq;
use embassy_time::Timer;

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use panic_probe as _;

use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Config, FifoJoin, InterruptHandler, Pio, ShiftConfig, ShiftDirection};

use fixed::types::U24F8;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});


const fn pack_grb(r: u8, g: u8, b: u8) -> u32 {
    ((g as u32) << 24) | ((r as u32) << 16) | ((b as u32) << 8)
}

const CYCLES_PER_BIT: u32 = 10;

const NUM_LEDS: usize = 12;

const COLORS: [u32; 12] = [
    pack_grb(255, 0, 0),     // Red
    pack_grb(0, 255, 0),     // Green
    pack_grb(0, 0, 255),     // Blue
    pack_grb(255, 255, 0),   // Yellow
    pack_grb(255, 0, 255),   // Magenta
    pack_grb(0, 255, 255),   // Cyan
    pack_grb(255, 128, 0),   // Orange
    pack_grb(128, 0, 255),   // Purple
    pack_grb(255, 255, 255), // White
    pack_grb(255, 20, 147),  // Pink
    pack_grb(0, 255, 128),   // Spring Green
    pack_grb(255, 215, 0),   // Gold
];

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

        .wrap_target
        bit_loop:
            set pins, 0 [1]
            out x, 1
            jmp !x do_zero
        do_one:
            set pins, 1 [4]
            jmp bit_loop
        do_zero:
            set pins, 1 [2]
            set pins, 0 [2]
        .wrap
        "
    );

    let mut cfg = Config::default();
    cfg.use_program(&common.load_program(&prg.program), &[]);

    cfg.set_set_pins(&[&out_pin]);

    let ws2812_freq = U24F8::from_num(800);
    let bit_freq = ws2812_freq * CYCLES_PER_BIT;

    let clock_freq = U24F8::from_num(clk_sys_freq() / 1000);
    cfg.clock_divider = clock_freq / bit_freq;

    // FIFO config
    cfg.fifo_join = FifoJoin::TxOnly;

    cfg.shift_out = ShiftConfig {
        auto_fill: true,
        threshold: 24,
        direction: ShiftDirection::Left,
    };

    sm0.set_config(&cfg);

    sm0.set_enable(true);

    loop {
        for i in 0..NUM_LEDS {
            sm0.tx().wait_push(COLORS[i]).await;
        }

        Timer::after_millis(100).await;
    }
}
