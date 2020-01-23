build:
	cargo build

debug:
	cargo run

flash: build
	openocd -f interface/stlink-v2-1.cfg -f target/stm32f3x.cfg -c "program target/thumbv7em-none-eabihf/debug/usnake verify reset exit"

erase:
	st-flash erase
