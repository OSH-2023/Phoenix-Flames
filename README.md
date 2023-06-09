We are
# Phoenix-Flames
A group of OSH course

<img src="docs/src/symbol.png" alt="symol" style="zoom: 33%;" />

members:
* 邓博文
* 蒋文浩
* 王志成
* 晏铭
* 钟睿智

### 选题： 基于`SeL4`与`Rust`的操作系统重构与优化


### Schedule

| Time       | Theme                                                        |
| ---------- | ------------------------------------------------------------ |
| 3.16       | 组员初次讨论，提供一些思路与想法                             |
| 3.23, 3.30 | 广泛搜集了一些有关操作系统的内容，包含微内核，数据库管理，分布式系统等方面，主要偏重于概念相关的内容 |
| 4.6        | 确定目标为重构操作系统，可能使用`Rust`语言或`C`语言，基于目前已有的性能较为良好的`SeL4`操作系统，进行对某些方面的改进 |
| 4.8        | 确定选题方向为基于`SeL4`与`Rust`的操作系统重构与优化         |
| 4.13       | 邀请老师参与，讨论了此前调研报告的一些不足，调研的广度不够。同时明确了具体的选题内容与一些细节。 |
| 4.20       | 进行可行性报告的分工与设计，明确可行性报告的方向                                                             |
| 4.27       | 确立Rust学习进度提示以及初步改写方案                                                             |
| 5.11       | 交流Rust学习成果并共同探讨seL4的具体结构                                                             |
| 5.18       | 完成Sel4的初次部署，继续探索有关seL4的实现逻辑和运行方式                                                            |
| 5.25       | 初步了解seL4的运行方式，较为独立地进行了初步分工与进度安排                                                            |
| 6.1        | 共同探讨Lab4，确立为Ray的部署和测试并明确分工                                                             |
| 6.8        | 小组开会完成Lab4，分布式部署碰到了一些问题bug，故延期                                                             |
| 6.15       | 完成Ray的分布式部署，编写结题报告                                                             |
| 6.22       | 考前最后一次会议，交流大家各自改写部分的情况，进行同步，并明确考后的工作路线                                                            |
| 7.4        | 考后第一次会议，继续完善已经编写的Rust代码并尝试编译，探讨未来几天的编写与结题工作                                                            |


### 项目概述

sel4是一个开源的、高安全性、高性能的操作系统微内核。它的独特之处在于其全面的形式化验证，同时不影响它的高性能。
我们的目的是将C语言编写的sel4微内核使用Rust语言重构，以此增加其可维护性与可拓展性。
