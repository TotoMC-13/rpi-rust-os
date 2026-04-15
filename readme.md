# rpi-rust-os

Minimal kernel for Raspberry Pi 3 (AArch64) written in Rust.

## Requirements

- Rust nightly (`aarch64-unknown-none` target)
- QEMU (`qemu-system-aarch64`)

## Run

```bash
cargo run
```

## Resources

- [OS-DEV Wiki - Raspberry Pi Bare Bones](https://wiki.osdev.org/Raspberry_Pi_Bare_Bones)
- [BCM2835-ARM-Peripherals](https://www.raspberrypi.org/app/uploads/2012/02/BCM2835-ARM-Peripherals.pdf)
- [Mailbox Property Interface](https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface)
