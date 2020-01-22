target remote :3333
set print asm-demangle on
set print pretty on
set backtrace limit 32

# Detect unhandled exceptions, hard faults and panics
# NB: In the currently configured crate, panic-itm
# is used to print panic messages out through ITM
# break DefaultHandler
# break HardFault

# Set ITM
monitor tpiu config internal itm.txt uart off 8000000
monitor itm port 0 on

load

layout src
break idlescreen
continue
