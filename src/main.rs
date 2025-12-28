#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{Config as SpiConfig, Spi};
use embassy_stm32::{Config, time::Hertz};
use embassy_time::Delay;

// Graphics Imports
use core::fmt::Write;
use embassy_stm32::rcc::*;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, text::Text};
use embedded_hal_bus::spi::ExclusiveDevice;
use heapless::String;
use mipidsi::interface::SpiInterface;
use mipidsi::{Builder, models::ST7789};
use profont::PROFONT_24_POINT;

// Panic handler
use {defmt_rtt as _, panic_probe as _};

const HSE_FREQUENCY: u32 = 12_000_000;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut c = Config::default();

    c.rcc.hse = Some(Hse {
        freq: Hertz(HSE_FREQUENCY),
        mode: HseMode::Oscillator,
    });
    c.rcc.pll = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV3,
        mul: PllMul::MUL85,
        divp: Some(PllPDiv::DIV2),
        divq: Some(PllQDiv::DIV2),
        divr: Some(PllRDiv::DIV2),
    });
    c.rcc.sys = Sysclk::PLL1_R;
    c.rcc.boost = true;

    let p = embassy_stm32::init(c);

    info!("Starting ST7789 Example...");

    let mut spi_config = SpiConfig::default();
    spi_config.frequency = Hertz(40_000_000);
    spi_config.mode = embassy_stm32::spi::MODE_3;

    let spi = Spi::new_txonly(
        p.SPI1, p.PA5,      // SCK
        p.PA7,      // MOSI
        p.DMA1_CH1, // TX DMA
        spi_config,
    );

    let cs = Output::new(p.PA0, Level::Low, Speed::VeryHigh);
    let dc = Output::new(p.PB6, Level::High, Speed::VeryHigh);
    let rst = Output::new(p.PA6, Level::Low, Speed::VeryHigh);

    let spi_device = ExclusiveDevice::new_no_delay(spi, cs);

    let mut buffer = [0u8; 1024];

    let di = SpiInterface::new(spi_device, dc, &mut buffer);

    let mut display = Builder::new(ST7789, di)
        .display_size(240, 240)
        .reset_pin(rst)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .init(&mut Delay)
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    let style = MonoTextStyleBuilder::new()
        .font(&PROFONT_24_POINT)
        .text_color(Rgb565::WHITE)
        .background_color(Rgb565::BLACK)
        .build();

    let mut counter = 0;

    info!("Starting Loop...");

    loop {
        let mut text_buffer: String<32> = String::new();
        core::write!(text_buffer, "Count: {}", counter).unwrap();

        Text::new(&text_buffer, Point::new(40, 110), style)
            .draw(&mut display)
            .unwrap();

        counter += 1;
        embassy_time::Timer::after_secs(1).await;
    }
}
