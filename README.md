# `usnake` :snake:
[![Crates.io](https://img.shields.io/crates/v/usnake.svg)](https://crates.io/crates/usnake)
[![Build Status](https://travis-ci.org/arosspope/usnake.svg?branch=master)](https://travis-ci.org/arosspope/usnake)

A rust implementation of the game [snake](https://en.wikipedia.org/wiki/Snake_(video_game_genre)) for the stm32f3 discovery board.

tags: rtfm
categories = ["embedded", "hardware-support", "no-std"]
keywords = ["snake", "rtfm", "game", "embedded"]

## Introduction

![img-1](https://i.imgur.com/yKoJNrH.jpg) ![img-2](https://imgur.com/a/gMvj2Fx.gif)
> blah blah (link to video)

Using the stm32f3, an 8x8 LED display and an analog joystick, I implemented the game snake using Rust's real-time embedded framework for Cortex-M microcontrollers - Real Time For the Masses ([RTFM](https://github.com/rtfm-rs/cortex-m-rtfm)). This project was primarily a learning exercise in understanding how Rust can be used to solve some of the challenges inherent in embedded application development. It includes examples on how to:

- Initialise peripherals and interact with them (i.e. digital pins for display, and ADCs for the joystick).
- Use RTFM to orchestrate software tasks that share mutable resources (i.e. initialised peripherals).
- Write `macros!` to simplify repeated code patterns, in this case:
    * Logging messages through, and ensuring exclusive access to, Cortex's standard ITM peripheral.
    * Scheduling tasks based on the `sysclk` frequency and a desired delay (in seconds).

## Playing the game

The hardware required for play includes:
* [STM32F3DISCOVERY](https://www.st.com/en/evaluation-tools/stm32f3discovery.html)
* [MAX7219](https://core-electronics.com.au/max7219-serial-dot-matrix-display-module.html) LED display
* [Analog Joystick](https://www.jaycar.com.au/arduino-compatible-x-and-y-axis-joystick-module/p/XC4422)
* [_hardware schematic here_]()


Playing this game will require you to prepare your development environment for cross-compliation 

To build and flash this game, I would suggest following the tutorial [here](https://rust-embedded.github.io/discovery/03-setup/index.html) to prepare your development environment. Assuming one has the necessary tools installed, the contained Makefile can be used to flash the board:
```
$ make flash
```
Once the binary has been flashed the LED display will start to cycle through a binary pattern - this means the system is now ready for play. To start the game, click the joystick.

A short snippet of gameplay can be found [here]().


TODO: 
 - Fix openocd file to pause at init routine


## Credits

This project wouldn't have been possible with out the following crates and documentation resources:
- stm32f3 crate
- max7219 crate
- discovery docs
- rtfm docs

## License

All source code (including code snippets) is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.
