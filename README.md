wtftw
=====

Window Tiling For The Win. A tiling window manager written in Rust

![Screenshot](https://i.imgur.com/Pq03fLx.jpg)

## Status
[![Build Status](https://travis-ci.org/Kintaro/wtftw.svg?branch=master)](https://travis-ci.org/Kintaro/wtftw)

## Build

**Notice:** Wtftw is compiled against the latest nightlies, so make sure to have your *rustc* up to date.

To build it, just run

```
cargo build
```

## Testing

First, export the ./target/deps path to your LD_LIBRARY_PATH, **this is important!**.
Then, if you want to have your own custom config, create one in *~/.wtftw/config.rs*.
You can find an example config in *config/config.rs* in this repository.

For testing, install either **Xnest** or **Xephyr** on your system and run

```
Xephyr -screen 800x600 :1 &
DISPLAY=:1 ./target/wtftw &
DISPLAY=:1 thunar & (or whatever application you want to run)
```

or respectively

```
Xnest -geometry 800x600+0+0 :1 &
DISPLAY=:1 ./target/wtftw &
DISPLAY=:1 thunar &
```

## Installation

Compile it normally with **cargo build**, and then either use it with your .xinitrc
or your favorite display manager. If you want to configure it, take a look at the example config in
*config/*.

After the first start, the config needs to be placed in *~/.wtftw/src/config.rs*. Voila.

## Commands

In a default setting, the commands are hardcoded, but can be changed in your own config.

### Switch workspace
```
ALT+num
```

### Open terminal
```
ALT+SHIFT+Enter
```

### Run program

```
ALT+SHIFT+p
```
You'll need to install gmrun for this

## Tutorial

I will be making a tutorial series on how to write a window manager. A bit busy with my thesis
at the moment, but the first part is [here](https://kintaro.github.io/rust/window-manager-in-rust-01/)
