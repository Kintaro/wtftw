wtftw
=====

Window Tiling For The Win. A tiling window manager written in Rust

![GIF sample](http://i.imgur.com/9pFfOZj.gif)

## Build

To build it, just run

```
cargo build
```

## Testing

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
