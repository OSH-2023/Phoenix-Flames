*这里描述了一种手动编译sel4的途径，可以从kernel_all.c文件与4个汇编文件编译得到sel4内核*

### sel4手动编译方法
首先按[教程](https://docs.sel4.systems/GettingStarted)完成sel4test的编译和运行。
然后在sel4test下创建build文件夹，从sel4test/build-x86/kernel中复制kernel_all.c和linker.lds_pp文件到当前文件夹下，从sel4test/kernel/src/arch/x86下复制multiboot.S、从sel4test/kernel/src/arch/x86/64下复制trap.S、machine_asm.S、head.S到当前文件夹。现在build文件夹下有一个c文件、4个汇编文件以及1个链接脚本文件。

在build目录下，执行以下命令将汇编文件（.S）编译成对象文件（.o）：
```shell
gcc -I ../kernel/include -I ../kernel/include/64 -I ../kernel/libsel4/include -I ../build-x86/kernel/autoconf -I ../build-x86/kernel/gen_config -I ../kernel/include/arch/x86 -I ../kernel/include/arch/x86/arch/64 -I ../build-x86/kernel/generated -I ../kernel/libsel4/sel4_arch_include/x86_64 -I ../kernel/libsel4/arch_include/x86 -I ../kernel/include/plat/pc99 -I ../kernel/include/plat/pc99/plat/64 -I ../build-x86/kernel/gen_headers -Wa,--64  -D__KERNEL_64__ -march=nehalem -O2 -g -DNDEBUG -std=c99 -Wall -Werror -Wstrict-prototypes -Wmissing-prototypes -Wnested-externs -Wmissing-declarations -Wundef -Wpointer-arith -Wno-nonnull -nostdinc -ffreestanding -fno-stack-protector -fno-asynchronous-unwind-tables -fno-common -O2 -nostdlib -fno-pic -fno-pie -DDEBUG -g -ggdb -mcmodel=kernel -mno-mmx -mno-sse -mno-sse2 -mno-3dnow -c *.S 
```

执行以下命令将kernel_all.c文件编译成对象文件：
```shell
gcc -I ../kernel/include -I ../kernel/include/64 -I ../kernel/libsel4/include -I ../build-x86/kernel/autoconf -I ../build-x86/kernel/gen_config -I ../kernel/include/arch/x86 -I ../kernel/include/arch/x86/arch/64 -I ../build-x86/kernel/generated -I ../kernel/libsel4/sel4_arch_include/x86_64 -I ../kernel/libsel4/arch_include/x86 -I ../kernel/include/plat/pc99 -I ../kernel/include/plat/pc99/plat/64 -I ../build-x86/kernel/gen_headers -c kernel_all.c -m64  -D__KERNEL_64__ -march=nehalem -O2 -g -DNDEBUG -std=c99 -Wall -Werror -Wstrict-prototypes -Wmissing-prototypes -Wnested-externs -Wmissing-declarations -Wundef -Wpointer-arith -Wno-nonnull -nostdinc -ffreestanding -fno-stack-protector -fno-asynchronous-unwind-tables -fno-common -O2 -nostdlib -fno-pic -fno-pie -DDEBUG -g -ggdb -mcmodel=kernel -mno-mmx -mno-sse -mno-sse2 -mno-3dnow
```

执行以下命令用链接脚本linker.lds_pp进行链接得到kernel.elf：
```shell
gcc *.o -m64  -D__KERNEL_64__ -march=nehalem -O2 -g -DNDEBUG -D__KERNEL_64__ -march=nehalem  -Wl,-m,elf_x86_64  -static -Wl,--build-id=none -Wl,-n -O2  -nostdlib  -fno-pic  -fno-pie  -DDEBUG  -g  -ggdb  -mcmodel=kernel     -Wl,-T linker.lds_pp -o kernel.elf
```

执行以下命令将kernel.elf处理成QEMU可以运行的格式，得到kernel文件：
```shell
gcc *.o -m64  -D__KERNEL_64__ -march=nehalem -O2 -g -DNDEBUG -D__KERNEL_64__ -march=nehalem  -Wl,-m,elf_x86_64  -static -Wl,--build-id=none -Wl,-n -O2  -nostdlib  -fno-pic  -fno-pie  -DDEBUG  -g  -ggdb  -mcmodel=kernel     -Wl,-T linker.lds_pp -o kernel.elf
```

把kernel移动到sel4test/build-x86/images文件夹下，用QEMU进行模拟：
```shell
qemu-system-x86_64  -cpu Nehalem,-vme,+pdpe1gb,-xsave,-xsaveopt,-xsavec,-fsgsbase,-invpcid,+syscall,+lm,enforce -nographic -serial mon:stdio -m size=3G  -kernel kernel -initrd sel4test-driver-image-x86_64-pc99
```

可以看到我们手动编译的内核可以正常运行。
