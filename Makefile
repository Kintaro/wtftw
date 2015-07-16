test: test_core test_xlib build_all

test_core:
	cd core && cargo test

test_xlib:
	cd xlib && cargo test

build_all:
	cargo build

.PHONY: test_core test_xlib build_all
