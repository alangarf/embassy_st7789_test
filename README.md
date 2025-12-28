# ST7789 Display Test with Embassy

An embedded Rust application for the STM32G431KB microcontroller that demonstrates interfacing with an ST7789 LCD display using the [Embassy](https://embassy.dev/) async framework.

## Overview

This project displays a simple counter on a 240x240 ST7789 LCD screen, incrementing every second. It showcases:
- Async embedded Rust programming with Embassy
- SPI communication with DMA for high-performance display updates
- Graphics rendering using the `embedded-graphics` and `mipidsi` libraries
- Real-time logging with `defmt`

## Hardware Requirements

- **Microcontroller**: STM32G431KB (Cortex-M4)
  - 12 MHz HSE (High-Speed External) oscillator
  - Configured to run at 170 MHz using PLL
- **Display**: ST7789 240x240 LCD (SPI interface)
- **Debugger**: probe-rs compatible debug probe (e.g., ST-Link)

### Pin Configuration

| Function | Pin | Description |
|----------|-----|-------------|
| SPI1 SCK | PA5 | SPI Clock |
| SPI1 MOSI | PA7 | SPI Data Out |
| SPI1 CS | PA0 | Chip Select (active low) |
| DC | PB6 | Data/Command control |
| RST | PA6 | Display Reset |

**SPI Configuration:**
- Mode: SPI Mode 3
- Frequency: 40 MHz
- DMA: Channel 1 for TX (transmit only)

## Features

- **High-Performance Graphics**: Utilizes DMA-accelerated SPI for fast display updates
- **Professional Text Rendering**: Uses ProFont 24-point font
- **Async/Await**: Built on Embassy's async executor for efficient resource usage
- **Debug Logging**: Runtime debug information via defmt and RTT

## Software Dependencies

Key dependencies include:
- `embassy-stm32`: STM32 HAL with async support
- `embassy-executor`: Async task executor
- `embassy-time`: Time and delay utilities
- `mipidsi`: Display driver for MIPI DBI/SPI displays
- `embedded-graphics`: 2D graphics library
- `defmt`: Efficient logging for embedded systems

## Building

### Prerequisites

1. Install Rust toolchain:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Add ARM Cortex-M4 target:
```bash
rustup target add thumbv7em-none-eabihf
```

3. Install probe-rs:
```bash
cargo install probe-rs-tools
```

### Build the Project

```bash
cargo build --release
```

## Running

Connect your STM32G431KB board via your debug probe and run:

```bash
cargo run --release
```

The application will:
1. Initialize the STM32G431 with a 170 MHz system clock
2. Configure SPI1 for 40 MHz communication
3. Initialize the ST7789 display
4. Clear the screen to black
5. Display a counter that increments every second

## Project Structure

```
.
├── Cargo.toml           # Project dependencies and configuration
├── build.rs             # Build script for linker configuration
├── .cargo/
│   └── config.toml      # Cargo and probe-rs runner configuration
├── Embed.toml           # Embedded tooling configuration
├── rust-toolchain.toml  # Rust toolchain specification
└── src/
    └── main.rs          # Main application code
```

## Code Highlights

- **Clock Configuration**: The STM32G431 is configured to run at 170 MHz using the PLL with HSE as source
- **Async Loop**: The main loop uses `embassy_time::Timer` for non-blocking 1-second delays
- **Text Rendering**: Demonstrates using `embedded-graphics` with heap-less string formatting

## License

This is a test/example project. Check individual dependency licenses for their terms.
