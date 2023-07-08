#! /usr/bin/bash
cd build-rust
qemu-system-x86_64  -cpu Nehalem,-vme,+pdpe1gb,-xsave,-xsaveopt,-xsavec,-fsgsbase,-invpcid,+syscall,+lm,enforce -nographic -serial mon:stdio -m size=3G  -kernel kernel -initrd sel4test-driver-image-x86_64-pc99
