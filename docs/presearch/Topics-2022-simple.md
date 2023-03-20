## 选题调研(simple)

------

**Format:**

#### Name	...	**Topic **	...	**Tags **	...	**Target ** ...

> Base&Reference

**language ...**

------



### 从前的课程选题——2022

#### WowKiddy

**Topic: distributed dataset platform**

**Tags: `Distributed platform`, `JuiceFS`, `Frontend`**

**Target: dataset platform for shared images and videos**

> Base&Reference:
>
> DisGraFs--------------among 2021 topics
>
> [JuiceFS](https://github.com/juicedata/juicefs)--------------(open-source) distributed file system
>
> [Neo4j](https://github.com/neo4j/neo4j)-----------------Graph Database
>
> [Prometheus](https://github.com/prometheus/prometheus)--------monitor the system
>
> [Grafana](https://github.com/grafana/grafana) -------------monitor the system
>
> [Docker](https://github.com/docker/compose)---------------deploy

**Language: 含有一些前端内容，另外所用的主要语言是Python**



#### Runikraft

**Topic: unikernel**

**Tags: `RISC-V`, `QEMU`, `microkernel`**

**Target: Use Rust to rewrite unikernel and realize some specified functions**

> Base&Reference:
>
> platform-----------------QEMU
>
> RISC-V architecture virtialization

**Language: Rust & C**



####x-QvQ

**Topic: RaspiOS (3b+)**

**Tags: `RaspiOS`, `ARM`, `Hardware`**

**Target: Run raspiOS**

> Base&Reference:
>
> RedLeaf-------------------Operation System by Rust company
>
> rCore-----------------------microkernel
>
> [DisGraFS](https://github.com/OSH-2021/x-DisGraFS/blob/main/docs/final_report/conclusion.md)-----------------among 2021 topics

> other repos for reference:
>
> [raspberrypi / linux (github.com)](https://github.com/raspberrypi/linux)
>
> [s-matyukevich / raspberry-pi-os (github.com)](https://github.com/s-matyukevich/raspberry-pi-os)

Language: Rust



####VR-fancy-office

**Topic: VR office**

**Tags: `VR`, `3D`, `Oculus`, `Remote Desktop`**

**Target:**

> Base&Reference:
>
> Oculus（VR耳机）应用开发
>
> Oculus integration SDK-------组件库
>
> X Protocol------------------------类 Unix 系统上的可视化协议
>
> --------------------------------------and many other protocols
>
> rustdesk,freerdp----------------一些远程桌面软件

language: Python3 (with socket, pynput, win32api) (侧重应用，有许多外部库等，不太能辨别清楚)



####x-NooBirds

**Topic: applications of real-time OSes**

**Tags: `RaspiOS`, `real-time`, `Hardware`, `Cloud control`**

**Target: 树莓派小车与智能交通系统**

> Base&Reference:
>
> Real-Time Linux
>
> CarSim

language: Python (mainly)



#### x-realism

**Topic: 构建操作系统**

**Tags: `Rust`, `Build OS`, `microkernel`**

**Target: 注重性能、并发和安全的微内核操作系统**

> Base&Reference:
>
> BlogOS
>
> rCore
>
> seL4
>
> 这一组调研时方向就已经很明确，直接开始说细节了...

language: Rust & C



#### x-DelayNoMore

**Topic: 分布式集群耦合适配深度学习模型**

**Tags: `Distributed System`**

**Target: 用 Ray 实现任务部署,信息在ROS，Ray 和深度学习框架高效传递整合**

> Base&Reference:
>
> ROS 2----------------------Robot Operating System
>
> DDS-------------------------Data Distribution Service
>
> Ray--------------------------集群计算框架

language: Python, shell



####x-TOBEDONE

**Topic: 分布式系统的完善**

**Tags: `x-DisGraFS`, `Distributed System`, `Monitor`**

**Target: 优化DisGraFS**

> Base&Reference:
>
> DisGraFS
>
> Prometheus 
>
> dontpanic
>
> CAT等监控系统
>
> Ray

language: Javascript等(写页面的那几个语言), python, Java(?)



####x_do_our_best

**Topic: 实时嵌入式**

**Tags: `嵌入式`, `ROS`， `RT-Thread`，`STM32`**

**Target: STM32控制小车运动**

> Base&Reference:
>
> ROS
>
> RT-Thread

language: C
