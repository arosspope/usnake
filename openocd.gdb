target remote :3333
set print asm-demangle on
set print pretty on
set backtrace limit 32

# Detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
# break rust_begin_unwind

# Set ITM
monitor tpiu config internal itm.txt uart off 8000000
monitor itm port 0 on

load

layout src
break idle
continue
