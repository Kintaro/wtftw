wtftw
=====

[![Gitter](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/Kintaro/wtftw?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

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

If you want to have your own custom config, create one in *~/.wtftw/src/config.rs*.
You can find an example config in *config/config.rs* in this repository.

For testing, install either **Xnest** or **Xephyr** on your system and run

```
Xephyr -screen 800x600 :1 &
DISPLAY=:1 ./target/release/wtftw &
DISPLAY=:1 thunar & (or whatever application you want to run)
```

or respectively

```
Xnest -geometry 800x600+0+0 :1 &
DISPLAY=:1 ./target/release/wtftw &
DISPLAY=:1 thunar &
```

## Installation

Compile it normally with **cargo build --release**, and then either use it with your .xinitrc
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

## FAQ

#### Does it work with dual monitors?

Yes, yes it does. Just use xrandr and you're set. Wtftw will automatically detect the changed setup

#### What are the alternatives to xmobar?

Dzen

#### what font and programs are you using on the screenshot?

The font is Envy Code R, and the programs are xmobar, ranger (file manager), vim and pms (mpd client)

#### why should I use wtftw than dwm or even awesome?

That is more of a personal choice. Wtftw is akin to xmonad. You can do almost anything you want with the config file. Extend it, change it at runtime, your only boundary is the rust language itself. Plus, using it would help a Rust project to detect bugs and improve it.

## Tutorial

I will be making a tutorial series on how to write a window manager. A bit busy with my thesis
at the moment, but the first part is [here](https://kintaro.github.io/rust/window-manager-in-rust-01/)
