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
use mipidsi::interface::{Interface, SpiInterface};
use mipidsi::{Builder, models::ST7789};
use profont::PROFONT_24_POINT;

const PORCTRL: u8 = 0xB2;
const GCTRL: u8 = 0xB7;
const VCOMS: u8 = 0xBB;
const LCMCTRL: u8 = 0xC0;
const VDVVRHEN: u8 = 0xC2;
const VRHS: u8 = 0xC3;
const VDVS: u8 = 0xC4;
const FRCTRL2: u8 = 0xC6;
const PWCTRL1: u8 = 0xD0;
const PVGAMCTRL: u8 = 0xE0;
const NVGAMCTRL: u8 = 0xE1;

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

    // 3. Initialize the chip with this high-speed config
    let p = embassy_stm32::init(c);

    info!("Starting ST7789 Example...");

    // --- 1. Setup SPI ---
    // High speed is important for displays!
    // The ST7789 can usually handle 40-50MHz easily.
    let mut spi_config = SpiConfig::default();
    spi_config.frequency = Hertz(40_000_000);
    spi_config.mode = embassy_stm32::spi::MODE_3;

    // Initialize SPI1 using DMA (Stream 1 for TX)
    // We don't need MISO (rx_dma) for a display, so we pass 'NoDma'
    let spi = Spi::new_txonly(
        p.SPI1, p.PA5,      // SCK
        p.PA7,      // MOSI
        p.DMA1_CH1, // TX DMA
        spi_config,
    );

    // --- 2. Setup Control Pins ---
    // CS (Chip Select) - Active Low
    let cs = Output::new(p.PA0, Level::Low, Speed::VeryHigh);
    // DC (Data/Command)
    let dc = Output::new(p.PB6, Level::High, Speed::VeryHigh);
    // RST (Reset)
    let rst = Output::new(p.PA6, Level::Low, Speed::VeryHigh);

    // --- 3. Create Display Interface ---
    let spi_device = ExclusiveDevice::new_no_delay(spi, cs);

    let mut buffer = [0u8; 1024];

    let di = SpiInterface::new(spi_device, dc, &mut buffer);

    // --- 4. Initialize the Driver (mipidsi) ---
    // ST7789, 240x240 resolution
    // We pass 'embassy_time::Delay' so the driver can wait during reset
    let mut display = Builder::new(ST7789, di)
        .display_size(240, 240)
        .reset_pin(rst)
        .invert_colors(mipidsi::options::ColorInversion::Inverted) // ST7789 often needs this
        .init(&mut Delay)
        .unwrap();

    let dcs = unsafe { display.dcs() };

    dcs.send_command(PORCTRL, &[0x0c, 0x0c, 0x00, 0x33, 0x33])
        .unwrap();
    dcs.send_command(GCTRL, &[0x35]).unwrap();
    dcs.send_command(VCOMS, &[0x19]).unwrap();
    dcs.send_command(LCMCTRL, &[0x2C]).unwrap();
    dcs.send_command(VDVVRHEN, &[0x01]).unwrap();
    dcs.send_command(VRHS, &[0x12]).unwrap();
    dcs.send_command(VDVS, &[0x20]).unwrap();
    dcs.send_command(FRCTRL2, &[0x0F]).unwrap();
    dcs.send_command(PWCTRL1, &[0xA4, 0xA1]).unwrap();
    dcs.send_command(
        PVGAMCTRL,
        &[
            0xD0, 0x04, 0x0D, 0x11, 0x13, 0x2B, 0x3F, 0x54, 0x4C, 0x18, 0x0D, 0x0B, 0x1F, 0x23,
        ],
    )
    .unwrap();
    dcs.send_command(
        NVGAMCTRL,
        &[
            0xD0, 0x04, 0x0C, 0x11, 0x13, 0x2C, 0x3F, 0x44, 0x51, 0x2F, 0x1F, 0x1F, 0x20, 0x23,
        ],
    )
    .unwrap();

    // --- 5. Draw Directly to Screen ---

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
