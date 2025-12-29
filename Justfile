# Justfile
set shell := ["bash", "-c"]

# Run the reset command
reset:
    probe-rs reset --chip STM32G431KB

# Flash and Reset
flash:
    cargo flash --release --chip STM32G431KB

build:
    cargo build

run:
    cargo run

# Open the GUI debugger
debug:
    probe-rs debug --chip STM32G431KB
