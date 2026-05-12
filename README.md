# CurrentMeter

Rust workspace for a `CurrentMeter` project targeting **STM32L052C8Tx** with the
**Embassy** async embedded ecosystem.

## Workspace layout

- `currentmeter-core`: hardware-independent domain logic designed to compile on
  both embedded targets and x64 hosts. This crate is where current-processing
  and algorithmic logic should live so it can be tested on a desktop PC.
- `currentmeter-firmware`: hardware-specific Embassy firmware for
  `thumbv6m-none-eabi` / STM32L052C8Tx.

## Common commands

Run host-side tests for the shared crate:

```bash
cargo host-test
```

Type-check embedded firmware:

```bash
cargo fw-check
```

Run firmware on hardware with probe-rs:

```bash
cargo firmware
```

## Notes

- The root `.cargo/config.toml` defaults Cargo to the MCU target. Override the
  target for host work with `--target x86_64-pc-windows-msvc` when needed.
- `embassy-stm32` is configured for the `stm32l052c8` device feature.