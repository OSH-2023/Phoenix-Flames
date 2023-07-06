## 构建方法

首先根据[官方教程](https://docs.sel4.systems/GettingStarted)完成sel4的编译和运行。在sel4test/build-x86/images文件夹下，可以看到两个文件：kernel-x86_64-pc99与sel4test-driver-image-x86_64-pc99，其中前者是内核的镜像文件，后者是测试程序。这里要编译出的就是我们的内核文件kernel，结合sel4test-driver-image-x86_64-pc99就可以在QEMU上进行模拟。

确保已经安装Rust，执行以下命令切换到nightly版的Rust：
```
rustup install nightly
rustup default nightly
```

执行以下命令添加一个组件：
```
rustup add target x86_64-unknown-none
```

现在转到src/sel4-Rust/sel4-Rust文件夹下，执行以下命令进行编译：
```
cargo build --release
```
在src/sel4-Rust/sel4-Rust/target/x86_64-unknown-none/release中找到编译出的静态库`libsel4_rust.a`，将该文件复制到src/sel4-Rust/build-rust文件夹下，然后在该文件夹下，
执行以下命令将汇编文件（.S）编译成对象文件（.o）：
```
gcc -I ../kernel/include -I ../kernel/include/64 -I ../kernel/libsel4/include -I ../build-x86/kernel/autoconf -I ../build-x86/kernel/gen_config -I ../kernel/include/arch/x86 -I ../kernel/include/arch/x86/arch/64 -I ../build-x86/kernel/generated -I ../kernel/libsel4/sel4_arch_include/x86_64 -I ../kernel/libsel4/arch_include/x86 -I ../kernel/include/plat/pc99 -I ../kernel/include/plat/pc99/plat/64 -I ../build-x86/kernel/gen_headers -Wa,--64  -D__KERNEL_64__ -march=nehalem -O2 -g -DNDEBUG -std=c99 -Wall -Werror -Wstrict-prototypes -Wmissing-prototypes -Wnested-externs -Wmissing-declarations -Wundef -Wpointer-arith -Wno-nonnull -nostdinc -ffreestanding -fno-stack-protector -fno-asynchronous-unwind-tables -fno-common -O2 -nostdlib -fno-pic -fno-pie -DDEBUG -g -ggdb -mcmodel=kernel -mno-mmx -mno-sse -mno-sse2 -mno-3dnow -c *.S 
```

执行以下命令将kernel_all.c编译成对象文件：

```
gcc -I ../kernel/include -I ../kernel/include/64 -I ../kernel/libsel4/include -I ../build-x86/kernel/autoconf -I ../build-x86/kernel/gen_config -I ../kernel/include/arch/x86 -I ../kernel/include/arch/x86/arch/64 -I ../build-x86/kernel/generated -I ../kernel/libsel4/sel4_arch_include/x86_64 -I ../kernel/libsel4/arch_include/x86 -I ../kernel/include/plat/pc99 -I ../kernel/include/plat/pc99/plat/64 -I ../build-x86/kernel/gen_headers -c kernel_all.c -m64  -D__KERNEL_64__ -march=nehalem -O2 -g -DNDEBUG -std=c99 -Wall -Werror -Wstrict-prototypes -Wmissing-prototypes -Wnested-externs -Wmissing-declarations -Wundef -Wpointer-arith -Wno-nonnull -nostdinc -ffreestanding -fno-stack-protector -fno-asynchronous-unwind-tables -fno-common -O2 -nostdlib -fno-pic -fno-pie -DDEBUG -g -ggdb -mcmodel=kernel -mno-mmx -mno-sse -mno-sse2 -mno-3dnow
```
将编译得到的对象文件和我们的静态库进行链接得到ELF格式的可执行文件kernel.elf：
```
gcc *.o -L. -lsel4_rust -m64  -D__KERNEL_64__ -march=nehalem -O2 -g -DNDEBUG -D__KERNEL_64__ -march=nehalem  -Wl,-m,elf_x86_64  -static -Wl,--build-id=none -Wl,-n -O2  -nostdlib  -fno-pic  -fno-pie  -DDEBUG  -g  -ggdb  -mcmodel=kernel     -Wl,-T linker.lds_pp -o kernel.elf
```

将ELF格式可执行文件转换成QEMU可以运行的内核镜像文件kernel：
```
objcopy -O elf32-i386 kernel.elf kernel
```

在之前编译运行成功的sel4test的文件夹中，找到sel4test-driver-image-x86_64-pc99，将它放到这个文件夹下。
执行以下命令运行测试程序：
```
qemu-system-x86_64  -cpu Nehalem,-vme,+pdpe1gb,-xsave,-xsaveopt,-xsavec,-fsgsbase,-invpcid,+syscall,+lm,enforce -nographic -serial mon:stdio -m size=3G  -kernel kernel -initrd sel4test-driver-image-x86_64-pc99
```
可以看到我们的内核镜像运行测试程序的输出与sel4内核的输出相同。





