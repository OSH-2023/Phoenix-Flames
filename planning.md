# sel4改写规划

## 改写分工
* Capability Spaces：蒋文浩
* Message Passing (IPC)：王志成
* Notifications：钟睿智
* Threads and Execution：晏铭
* Address Spaces and Virtual
Memory：邓博文

## 开发环境的构建
在rCore教程[实验环境配置](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/5setup-devel-env.html#)的一页中，完成“Rust 开发环境配置”的部分。

## 改写注意事项
### 1. 改写原则
目前的计划是尽量使用Rust完全重构sel4，所以要尽量多地使用Rust语言的特性，尽量避免使用unsafe块，避免把Rust程序写的跟C程序一样。比如，不要用裸指针而是用引用，全局变量不要用static mut和unsafe，而是用下面7.中提到的方法。
### 2. 文件改写的协作
每次修改一个文件后，**在文件开头注释上自己的名字和日期**，比如：
```rust
//DBW 6.9 12:38
```
**为了避免多个人修改同一份文件造成的冲突**，建议采取这样的方法：把git仓库再复制一份，修改文件时就在这份复制的仓库修改，要提交时，先在原本的仓库进行一次git pull，查看是否有更新，如果没有更新或者更新与自己修改的文件无关，那么将自己修改了的文件直接复制进这个仓库，然后提交；如果刚好有人修改过自己刚才也修改了的文件，那么就打开这个文件，将自己刚才写的内容添加进去，然后再提交。
### 3. 结构体的改写
比如源代码中有这样一个结构体：
```C
struct cte {
    cap_t cap;
    mdb_node_t cteMDBNode;
};
typedef struct cte cte_t;
```
这个结构体在object/structures.h中定义。现在你需要添加这个结构体到我们的sel4-Rust中。目前大部分源代码的文件在我们的项目中都已经有了相应的位置，注意要放到准确的位置。现在，你发现sel4-Rust中有一个叫做object.rs的文件，似乎与这个头文件有关，而且你在这个文件开头看到了这样的注释：
```rust
/*
Including contents from:
1. object/structures.h
2. arch/object/structures.h
3. arch/object/structures_gen.h
 */
```
也就是说这三个头文件都对应这一个.rs文件，其中就有object/structures.h，于是你应该把改写的结构体就放在这个文件里。在这个文件里找到这样一行注释（如果没有则创建）：
```rust
//1. from object/structures.h
```
所有来自这个头文件的内容都应该放在这之后。现在你将这个结构体改写为Rust形式：
```rust
#[derive(Clone)]
pub struct cte {
    pub cap: cap_t,
    pub cteMDBNode: mdb_node_t,
}
pub type cte_t = cte;
```
注意每一行都有一个pub。注意这里只实现了Clone trait，没有实现Copy，这是为了防止不必要的复制带来的性能损失。
### 4. 函数的改写
函数定义前先不用加extern "C"，因为不一定需要与C交互。
### 5. .c文件的改写
.c文件一般有与其对应的.h文件，比如cspace.c和cspace.h，在这种情况下，sel4-Rust中对应的就是同名的.rs文件，比如cspace.rs，它同时包含cspace.h和cspace.c的内容。
### 6. 禁用一些命名的警告
由于结构体和变量命名不符合Rust标准，编译器会给出警告，在文件开头加上这样一段可以禁用这种警告：
```rust
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
```
### 7. 全局变量的处理方式
在Rust中对全局变量的每一次使用都会用到unsafe块，这是非常糟糕的（否则用Rust改写就没意义了）。[rCore的教程](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter2/3batch-system.html)中给出了一种解决方案，目前sel4-Rust中已经包含了相关代码。如果要创建一个全局变量，你需要这样：
```rust
use crate::sync::UPSafeCell;
use lazy_static::lazy_static;

lazy_static! {
    //声明变量
    static ref global_variable: UPSafeCell<ClassName> = ...;
}
```
然后在需要使用的时候，你需要这样：
```rust
let mut use_global_variable = global_variable.exclusive_access();
//使用use_global_variable，你可以像一个ClassName类型的可变借用一样使用它
drop(use_global_variable);
//用完了需要执行这一条
```


