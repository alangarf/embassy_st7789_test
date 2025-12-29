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

// Dummy pin for CS
use core::convert::Infallible;
use embedded_hal::digital::{ErrorType, OutputPin};

// Panic handler
use {defmt_rtt as _, panic_probe as _};

const HSE_FREQUENCY: u32 = 12_000_000;
const SPI_FREQUENCY: u32 = 40_000_000;
const DISPLAY_WIDTH: u16 = 240;
const DISPLAY_HEIGHT: u16 = 240;
const SPI_INTERFACE_BUFFER_SIZE: usize = 1024;

fn setup_sys_clock() -> Config {
    let mut cfg = Config::default();

    // setup sysclk to use HSE and generate 170MHz
    cfg.rcc.hse = Some(Hse {
        freq: Hertz(HSE_FREQUENCY),
        mode: HseMode::Oscillator,
    });
    cfg.rcc.pll = Some(Pll {
        source: PllSource::HSE,
        prediv: PllPreDiv::DIV3,
        mul: PllMul::MUL85,
        divp: Some(PllPDiv::DIV2),
        divq: Some(PllQDiv::DIV2),
        divr: Some(PllRDiv::DIV2),
    });
    cfg.rcc.sys = Sysclk::PLL1_R;
    cfg.rcc.boost = true;

    cfg
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(setup_sys_clock());

    info!("Starting ST7789 Example...");

    // the SPI buffer to allow efficient transfers to the display
    // large equals faster but more RAM, smaller less RAM, less efficient
    let mut buffer = [0u8; SPI_INTERFACE_BUFFER_SIZE];

    // configure chip select, data/command and reset pins
    // fake the cs pin if single display or not needed
    #[cfg(feature = "real_cs")]
    let cs = Output::new(p.PA0, Level::Low, Speed::Medium);
    #[cfg(not(feature = "real_cs"))]
    let cs = DummyCsPin;

    let dc = Output::new(p.PB6, Level::High, Speed::Medium);
    let rst = Output::new(p.PA6, Level::Low, Speed::Medium);

    // setup the SPI device
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = Hertz(SPI_FREQUENCY);
    spi_config.mode = embassy_stm32::spi::MODE_3;

    let spi = Spi::new_txonly(p.SPI1, p.PA5, p.PA7, p.DMA1_CH1, spi_config);
    let spi_device = ExclusiveDevice::new_no_delay(spi, cs);

    let di = SpiInterface::new(spi_device, dc, &mut buffer);

    // setup the display
    let mut display = Builder::new(ST7789, di)
        .display_size(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .reset_pin(rst)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .init(&mut Delay)
        .unwrap();

    // clear the screen to black
    display.clear(Rgb565::BLACK).unwrap();

    // make a nice large font
    let style = MonoTextStyleBuilder::new()
        .font(&PROFONT_24_POINT)
        .text_color(Rgb565::WHITE)
        .background_color(Rgb565::BLACK)
        .build();

    let mut counter = 0;
    let mut text_buffer: String<32> = String::new();

    info!("Starting Loop...");
    loop {
        text_buffer.clear();
        core::write!(text_buffer, "Count: {}", counter).unwrap();

        Text::new(&text_buffer, Point::new(10, 120), style)
            .draw(&mut display)
            .unwrap();

        counter += 1;
        embassy_time::Timer::after_millis(50).await;
    }
}

// A DummyCsPin satisfies the embedded-hal OutputPin requirement for
// ExclusiveDevice when a hardware CS pin is not physically used.
pub struct DummyCsPin;

impl ErrorType for DummyCsPin {
    type Error = Infallible;
}

impl OutputPin for DummyCsPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
