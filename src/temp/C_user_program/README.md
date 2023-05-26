## how to build:

```
# under src
riscv64-unknown-elf-gcc *.c -nostdlib  -mcmodel=medany -T linker.ld -o user_program
riscv64-unknown-elf-objcopy user_program -O binary user_program.bin
```