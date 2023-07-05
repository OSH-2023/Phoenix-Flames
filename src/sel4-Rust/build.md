How to compile?

```
gcc -I ../kernel/include -I ../kernel/include/64 -I ../kernel/libsel4/include -I ../build-x86/kernel/autoconf -I ../build-x86/kernel/gen_config -I ../kernel/include/arch/x86 -I ../kernel/include/arch/x86/arch/64 -I ../build-x86/kernel/generated -I ../kernel/libsel4/sel4_arch_include/x86_64 -I ../kernel/libsel4/arch_include/x86 -I ../kernel/include/plat/pc99 -I ../kernel/include/plat/pc99/plat/64 -I ../build-x86/kernel/gen_headers -Wa,--64  -D__KERNEL_64__ -march=nehalem -O2 -g -DNDEBUG -std=c99 -Wall -Werror -Wstrict-prototypes -Wmissing-prototypes -Wnested-externs -Wmissing-declarations -Wundef -Wpointer-arith -Wno-nonnull -nostdinc -ffreestanding -fno-stack-protector -fno-asynchronous-unwind-tables -fno-common -O2 -nostdlib -fno-pic -fno-pie -DDEBUG -g -ggdb -mcmodel=kernel -mno-mmx -mno-sse -mno-sse2 -mno-3dnow -c *.S 

gcc -I ../kernel/include -I ../kernel/include/64 -I ../kernel/libsel4/include -I ../build-x86/kernel/autoconf -I ../build-x86/kernel/gen_config -I ../kernel/include/arch/x86 -I ../kernel/include/arch/x86/arch/64 -I ../build-x86/kernel/generated -I ../kernel/libsel4/sel4_arch_include/x86_64 -I ../kernel/libsel4/arch_include/x86 -I ../kernel/include/plat/pc99 -I ../kernel/include/plat/pc99/plat/64 -I ../build-x86/kernel/gen_headers -c kernel_all.c -m64  -D__KERNEL_64__ -march=nehalem -O2 -g -DNDEBUG -std=c99 -Wall -Werror -Wstrict-prototypes -Wmissing-prototypes -Wnested-externs -Wmissing-declarations -Wundef -Wpointer-arith -Wno-nonnull -nostdinc -ffreestanding -fno-stack-protector -fno-asynchronous-unwind-tables -fno-common -O2 -nostdlib -fno-pic -fno-pie -DDEBUG -g -ggdb -mcmodel=kernel -mno-mmx -mno-sse -mno-sse2 -mno-3dnow

gcc *.o -L. -lsel4_rust -m64  -D__KERNEL_64__ -march=nehalem -O2 -g -DNDEBUG -D__KERNEL_64__ -march=nehalem  -Wl,-m,elf_x86_64  -static -Wl,--build-id=none -Wl,-n -O2  -nostdlib  -fno-pic  -fno-pie  -DDEBUG  -g  -ggdb  -mcmodel=kernel     -Wl,-T linker.lds_pp -o kernel.elf

objcopy -O elf32-i386 kernel.elf kernel
```

How to simulate?
```
qemu-system-x86_64  -cpu Nehalem,-vme,+pdpe1gb,-xsave,-xsaveopt,-xsavec,-fsgsbase,-invpcid,+syscall,+lm,enforce -nographic -serial mon:stdio -m size=3G  -kernel kernel -initrd sel4test-driver-image-x86_64-pc99
```






