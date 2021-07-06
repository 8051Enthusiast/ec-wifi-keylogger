These are the tools for reproducing the keylogger from [https://8051enthusiast.github.io/2021/07/05/002-wifi_fun.html](https://8051enthusiast.github.io/2020/04/14/003-Stream_Video_From_Mouse.html).
Requires an Lenovo 310-15IKB (or something else that has rtl8821ae and IT8586E with an rfkill line connected from the EC to the wifi card,
although this will need modifications depending on how it is connected).

For compilation, rust/cargo and the asem-51 assembler are required.

The realtek directory contains the source for the realtek firmware and new one can be built by running make.sh,
although you will need to set the MAC and IP addresses in packet.a51.
Then replace `/lib/firmware/rtlwifi/rtl8821aefw_29.bin` with the file `out` that was produced (when on linux).
Make sure the firmware file is loaded (for example, by reloading the kernel module).

The ecdebug directory contains the source of the patch to the EC firmware and a program to apply it.
Compile the program with `cargo build` and then run `sudo ./target/debug/ecdebug`.
There will be a prompt. Write `e write` and `p firmware/send.hex` to apply the patch.
To revert it, write `P` and `w 1627 80` (the second command sets the `EC_RX` pin to read mode again).
If this somehow messes up the state of the EC, you can reset it by pressing the power button for at least 10 seconds.

The `ps2udp_to_uinput` directory contains the receiver for the keylogging.
It needs to run on linux and any keys sent to it over port 10002 will be put into the X11 session.

Note that some programs here have a small chance of bricking your laptop, so do be careful.
