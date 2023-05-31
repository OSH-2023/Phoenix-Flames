*这里描述了一种可能的手动编译sle4的途径，尚未完全实现，但改进后可能会成功。*

### sel4手动编译方法
首先按教程完成sel4test的编译和运行。
然后在sel4test下创建build文件夹，从sel4test/build-x86/kernel中复制kernel_all.c和linker.lds_pp文件到当前文件夹下，从sel4test/kernel/src/arch/x86下复制multiboot.S、从sel4test/kernel/src/arch/x86/64下复制trap.S、machine_asm.S、head.S到当前文件夹。现在build文件夹下有一个c文件、4个汇编文件以及1个链接脚本文件。

执行以下命令将汇编文件（.S）编译成对象文件（.o）：
```
gcc -I ../kernel/include -I ../kernel/include/64 -I ../kernel/libsel4/include -I ../build-x86/kernel/autoconf -I ../build-x86/kernel/gen_config -I ../kernel/include/arch/x86 -I ../kernel/include/arch/x86/arch/64 -I ../build-x86/kernel/generated -I ../kernel/libsel4/sel4_arch_include/x86_64 -I ../kernel/libsel4/arch_include/x86 -I ../kernel/include/plat/pc99 -I ../kernel/include/plat/pc99/plat/64 -I ../build-x86/kernel/gen_headers -c *.S
```

执行以下命令将kernel_all.c文件编译成对象文件：
```
gcc -I ../kernel/include -I ../kernel/include/64 -I ../kernel/libsel4/include -I ../build-x86/kernel/autoconf -I ../build-x86/kernel/gen_config -I ../kernel/include/arch/x86 -I ../kernel/include/arch/x86/arch/64 -I ../build-x86/kernel/generated -I ../kernel/libsel4/sel4_arch_include/x86_64 -I ../kernel/libsel4/arch_include/x86 -I ../kernel/include/plat/pc99 -I ../kernel/include/plat/pc99/plat/64 -I ../build-x86/kernel/gen_headers -c kernel_all.c -m64  -ffreestanding  -nostdlib  -no-pie  -mno-red-zone  -mno-mmx  -mno-sse  -mno-sse2  -mno-sse3  -mno-3dnow -nostartfiles -nodefaultlibs
```

执行以下命令用链接脚本linker.lds_pp进行链接：
```
gcc *.o -T linker.lds_pp -o kernel -nostdlib -nodefaultlibs -no-pie
```

现在就得到了名为kernel的内核的ELF文件。使用file命令查看文件属性：
```
file kernel
```
可以看到输出为：
> kernel: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked, not stripped

把kernel移动到sel4test/build-x86/images文件夹下，尝试用QEMU进行模拟：
```
qemu-system-x86_64  -cpu Nehalem,-vme,+pdpe1gb,-xsave,-xsaveopt,-xsavec,-fsgsbase,-invpcid,+syscall,+lm,enforce -nographic -serial mon:stdio -m size=3G  -kernel kernel -initrd sel4test-driver-image-x86_64-pc99
```

但是很遗憾，当前这个文件还不能用于QEMU进行模拟。之后会继续探索编译出能够运行的内核文件的方法。
