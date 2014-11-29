wtftw
=====

Window Tiling For The Win. A tiling window manager written in Rust

![GIF sample](http://i.imgur.com/9pFfOZj.gif)

## Status
[![Build Status](https://travis-ci.org/Kintaro/wtftw.svg?branch=master)](https://travis-ci.org/Kintaro/wtftw)

## Build

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

## Commands

So far, the commands are hardcoded.

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
