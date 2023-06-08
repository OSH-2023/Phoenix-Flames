## Running steps:
```
#under sel4-Rust
cargo build --release
rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/sel4-Rust -O binary target/riscv64gc-unknown-none-elf/release/sel4-Rust.bin
qemu-system-riscv64 -machine virt -nographic -bios ./bootloader/rustsbi-qemu.bin -device loader,file=target/riscv64gc-unknown-none-elf/release/sel4-Rust.bin,addr=0x80200000 [-s -S]
#"-s -S" is used when debugging
```


