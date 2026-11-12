# Blackjack

A small desktop Blackjack game written in Rust with [egui](https://github.com/emilk/egui).

## Features

- Deal, hit, stand, double down, and split
- Editable wager and player balance
- Dealer stands on 17
- Blackjack pays 3:2
- Animated card dealing and a custom card-table interface

## Run from Cygwin on Windows

Install Rust with rustup, then install the Windows GNU toolchain once:

```bash
rustup toolchain install stable-x86_64-pc-windows-gnu
```

From the project directory, run the tests:

```bash
cargo +stable-x86_64-pc-windows-gnu test
```

Launch the game with:

```bash
cargo +stable-x86_64-pc-windows-gnu run
```

## Run from Windows Developer PowerShell

With the MSVC Rust toolchain and Visual Studio C++ build tools installed, run:

```powershell
cargo test
cargo run
```

## Controls

- **Deal / New Round:** Start a round using the selected wager
- **Hit:** Take another card
- **Stand:** End your turn
- **Double:** Double the wager and take one final card
- **Split:** Split a pair into two hands

The unit tests cover hand scoring, Blackjack detection, and wager limits.
