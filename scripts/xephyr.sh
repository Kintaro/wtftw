#!/bin/sh
Xephyr +extension RANDR -screen ${SCREEN_SIZE:-800x600} :1 -ac & (sleep 1; env DISPLAY=:1 ./target/release/wtftw & env DISPLAY=:1 termite)
