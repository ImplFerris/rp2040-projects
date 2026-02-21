#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

// defmt Logging
use defmt::info;
use defmt_rtt as _;

use panic_probe as _;

use embassy_rp::bind_interrupts;

use smart_leds::RGB8;

use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program};

/// Color orders for WS2812B, type RGB8
pub trait RgbColorOrder {
    /// Pack an 8-bit RGB color into a u32
    fn pack(color: RGB8) -> u32;
}

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[rustfmt::skip]
const COLORS: [RGB8; 12] = [
    RGB8 { r: 255, g: 0,   b: 0   }, // Red
    RGB8 { r: 255, g: 127, b: 0   }, // Orange
    RGB8 { r: 255, g: 255, b: 0   }, // Yellow
    RGB8 { r: 127, g: 255, b: 0   }, // Chartreuse
    RGB8 { r: 0,   g: 255, b: 0   }, // Green
    RGB8 { r: 0,   g: 255, b: 127 }, // Spring Green
    RGB8 { r: 0,   g: 255, b: 255 }, // Cyan
    RGB8 { r: 0,   g: 127, b: 255 }, // Azure
    RGB8 { r: 0,   g: 0,   b: 255 }, // Blue
    RGB8 { r: 127, g: 0,   b: 255 }, // Violet
    RGB8 { r: 255, g: 0,   b: 255 }, // Magenta
    RGB8 { r: 255, g: 0,   b: 127 }, // Rose
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("Initializing the program");

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.PIO0, Irqs);

    let program = PioWs2812Program::new(&mut common);
    let mut ws2812 = PioWs2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_15, &program);

    ws2812.write(&COLORS).await;

    loop {
        Timer::after_millis(100).await;
    }
}
