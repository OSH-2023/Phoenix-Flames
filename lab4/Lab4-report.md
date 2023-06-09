# OSH-2023 Phoneix-Flames Lab4实验报告

## 测试任务选定

本次选择的测试任务为转移概率矩阵的近似极限分布。

## 性能指标列表

* 吞吐量（Throughput）：指系统在单位时间内能够处理的请求或者任务数量，通常用每秒处理请求数（Requests per Second，RPS）或者每秒完成任务数（Tasks per Second，TPS）来衡量。

* 延迟（Latency）：指系统处理请求或者任务所需的时间，通常用平均响应时间（Average Response Time）或者百分位响应时间（Percentile Response Time）来衡量。
* 资源利用率（Resource Utilization）：指系统在运行过程中所使用的资源占总资源的比例，例如 CPU 使用率、内存使用率等。
* 可靠性（Reliability）：指系统在长时间运行中的稳定性和可靠性，通常用错误率（Error Rate）或者故障率（Failure Rate）来衡量。
* 扩展性（Scalability）：指系统在处理更多请求或者任务时的能力，通常用负载测试（Load Testing）或者压力测试（Stress Testing）来衡量。

我们选用任务延迟和吞吐量作为主要的关注点。

## 单机版部署及性能测试

#### 1、Ray的安装

##### 1、安装python:

```shell
sudo apt-get install python
```

##### 2、安装pip：

```shell
curl https://bootstrap.pypa.io/get-pip.py -o get-pip.py
sudo python3 get-pip.py
```

##### 3、更新：

```shell
sudo apt update
```

##### 4、安装ray：

```shell
pip install -U ray
pip install 'ray[default]'
```

#### 2、测试过程

##### 1、创建head结点

使用命令：

```shell
ray start --head
```

创建节点

##### 2、运行测试程序

例如测试程序命名为test.py且位于当前工作目录下，则运行如下命令来启动运行程序：

```
python test.py
```

##### 3、查看结果

运行创建节点命令之后，会出现如下内容:

```
To monitor and debug Ray, view the dashboard at 
    127.0.0.1:8265
```

在浏览器中输入该ip地址，即可打开*Dashboard*查看运行结果

##### 4、退出

输入命令：

```shell
ray stop
```

退出程序

#### 3、用python编写的ray部署测试代码(markov_new.py)

```python
import ray
import time
import sys
import numpy as np

ray.init()

matrix_size: int = 10    # 10 * 10 matrix
matrix_mul_times: int = 10000000
if len(sys.argv) < 1:
    pc_num: int = 10    # default value = 10
else:
    pc_num: int = int(sys.argv[1])
node_task_num: int = matrix_mul_times // pc_num


def matrix_mul(matrix1, matrix2):
    return np.matmul(matrix1, matrix2)


@ray.remote
class Worker(object):
    def __init__(self):
        self.size = matrix_size
        self.matrix0 = self.matrix_init2()
        self.res = self.matrix0.copy()
        self.all_times = matrix_mul_times

    def matrix_init2(self):
        # 每行均为正数浮点随机数，单行和为1（归一化）
        matrix = np.random.random((matrix_size, matrix_size))
        for row in range(self.size):
            total = sum(matrix[row])
            for item in matrix[row]:
                item /= total
        # print("Now we have a matrix\n", matrix)
        return matrix

    # 转移矩阵的极限分布
    # 计算若干个矩阵的相乘
    def calculate(self, times):
        # print("I start doing my work.")
        cur_time = time.time()
        task_res = self.matrix0.copy()
        for k in range(times-1):
            task_res = matrix_mul(self.res, self.matrix0)
        # print("I have finished my work, duration: ", time.time() - cur_time)
        return task_res


if __name__ == '__main__':
    cur_time=time.time()
    worker = Worker.remote()
    temps=[]
    for i in range(pc_num):
        temp = worker.calculate.remote(node_task_num)
        temps.append(temp)

    result_list = ray.get(temps)

    result = result_list[0]
    for m in result_list:
        result = matrix_mul(result, m)
    # print("final matrix: \n", result)
    print("total duration: ", time.time() - cur_time)
```

#### 4、初步测试（使用不含ray的同规格任务测试）

文件：`no_ray.py`

**单机参数**

USTC vlab虚拟机的标准配置。

##### 运行结果

![pic3](src/pic3.jpg)



#### 5、Ray单机版分析、测试、优化

##### 单机参数

（实体机linux ubuntu22.04）

```
(ray) tern@ubuntu:~/Documents/codes/python-codes/trial$ cat /proc/version
Linux version 5.19.0-43-generic (buildd@lcy02-amd64-028) (x86_64-linux-gnu-gcc (Ubuntu 11.3.0-1ubuntu1~22.04.1) 11.3.0, GNU ld (GNU Binutils for Ubuntu) 2.38) #44~22.04.1-Ubuntu SMP PREEMPT_DYNAMIC Mon May 22 13:39:36 UTC 2

(ray) tern@ubuntu:~/Documents/codes/python-codes/trial$ cat /proc/cpuinfo | grep "cpu cores" | uniq
cpu cores	: 14
```



##### 部署与运行过程（精简）

**程序参数**

```
matrix_size: int = 10    # 10 * 10 matrix
matrix_mul_times: int = 10000000
if len(sys.argv) < 1:
    pc_num: int = 10    # default value = 10
else:
    pc_num: int = int(sys.argv[1])
```

**部署与运行（simplified，详见rec.md）**

```bash
tern@ubuntu:~/Documents/codes/python-codes/trial$ conda init bash
modified      /home/tern/.bashrc

tern@ubuntu:~/Documents/codes/python-codes/trial$ source ~/.bashrc
(base) tern@ubuntu:~/Documents/codes/python-codes/trial$ conda activate ray
(ray) tern@ubuntu:~/Documents/codes/python-codes/trial$ ray start --head
    
(ray) tern@ubuntu:~/Documents/codes/python-codes/trial$ python markov_new.py 100000
2023-06-20 11:29:11,978	INFO worker.py:1342 -- Connecting to existing Ray cluster at address: 114.214.252.223:6379...
total duration:  27.68019461631775

(ray) tern@ubuntu:~/Documents/codes/python-codes/trial$ python markov_new.py 10000
2023-06-20 11:29:56,011	INFO worker.py:1342 -- Connecting to existing Ray cluster at address: 114.214.252.223:6379...
total duration:  11.307637929916382

(ray) tern@ubuntu:~/Documents/codes/python-codes/trial$ python markov_new.py 10
2023-06-20 11:30:27,501	INFO worker.py:1342 -- Connecting to existing Ray cluster at address: 114.214.252.223:6379...
total duration:  9.69461703300476

(ray) tern@ubuntu:~/Documents/codes/python-codes/trial$ python markov_new.py 1
2023-06-20 11:30:51,152	INFO worker.py:1342 -- Connecting to existing Ray cluster at address: 114.214.252.223:6379...
total duration:  9.28380298614502
```



**Dashboard**

![pic1](src/pic1.png)

![pic2](src/pic2.png)



**性能改进与提升**

![pic7](src/pic7.png)

**延迟测试：**

1. 总吞吐量30000000（执行30000000次矩阵乘法）：

| 任务分配数 | 100000 | 10000 | 10    | 1     |
| ---------- | ------ | ----- | ----- | ----- |
| 测试时间   | 46.26  | 30.71 | 28.08 | 26.96 |



(下一参数对应运行内容即上面的文本内容，记录在rec.md中)

2. 总吞吐量10000000：

| 任务分配数 | 100000 | 10000 | 10   | 1    |
| ---------- | ------ | ----- | ---- | ---- |
| 测试时间   | 27.68  | 11.31 | 9.70 | 9.28 |



![pic6](src/pic6.png)

3. 总吞吐量500000：

| 任务分配数 | 100000 | 10000 | 10   | 1    |
| ---------- | ------ | ----- | ---- | ---- |
| 测试时间   | 17.96  | 2.41  | 0.75 | 0.70 |



可见任务数增大时，时间开销大量增大。这是由于**进程间通讯**耗费了大量的时间。

对于第二种情况，即任务数为10000000时，最初任务拆分数为100000,运行效果很差，后来将只任务切分为10份，性能优化了`185%`以上。

同样的，对于第一种情况，即任务总数增加到30000000时，任务拆分数10相对于任务拆分数100000性能提高了`65%`。

而当任务总数降低到500000时，性能的提升就更为显著，主要是由于此时在拆分数100000的情形下每一个任务实际上只进行了五次矩阵乘法，而在合并运算结果时要进行100000次运算，因此并行性能极差，所以任务拆分数减少时性能的提升极为显著（超过五倍）。



**关于吞吐量：**

在固定任务拆分数为1时，

第一种情形下即任务总数为30000000时，吞吐率为1112759次运算/秒。

第二种情形下即任务总数为10000000时，吞吐率为1077586次运算/秒。

第三种情形下即任务总数为500000时，吞吐率为714286次运算/秒。



在固定任务拆分数为10时，

第一种情形下即任务总数为30000000时，吞吐率为1068376次运算/秒。

第二种情形下即任务总数为10000000时，吞吐率为1030928次运算/秒。

第三种情形下即任务总数为500000时，吞吐率为666667次运算/秒。



在固定任务拆分数为10000时，

第一种情形下即任务总数为30000000时，吞吐率为976880次运算/秒。

第二种情形下即任务总数为10000000时，吞吐率为884173次运算/秒。

第三种情形下即任务总数为500000时，吞吐率为207469次运算/秒。



在固定任务拆分数为100000时，

第一种情形下即任务总数为30000000时，吞吐率为648508次运算/秒。

第二种情形下即任务总数为10000000时，吞吐率为361272次运算/秒。

第三种情形下即任务总数为500000时，吞吐率为27840次运算/秒。



吞吐量随着任务数增多而增加，这一点可能是由于测试中的延迟时间包括合并任务时间，而这一处理是串行的，不可以并行执行，所以导致相对来说任务总数比较多时花费在合并上的时间较少，故而吞吐量较大。因而当拆分数较少时第一和第二种情形下的吞吐量十分相近。（这里在测第三种情形时因为出现了溢出的问题，所以对部分乘法加了一步归一化，导致单次运算执行时间变长了一些，；吞吐量有相应减少）

另外，在单机上，明显在任务拆分数较少的时候吞吐量表现较好，但当拆分数降到1000以下时差距已经很小，因为此时花费在合并任务上的时间已经相对较少，对吞吐量影响很小。



## 分布式部署及性能测试（基于docker）

这里使用了4台运行Ubuntu的虚拟机服务器（实际上就是vlab），由于处于同一局域网下，连接比较方便。
1台作为头结点和客户端，其余3台作为工作节点。

#### 1、准备结点：头节点和工作节点

##### 1、安装OpenSSH Server （Linux Ubuntu）

输入如下命令在ubuntu上安装和配置OpenSSH Server：

```shell
sudo apt update
sudo apt install openssh-server
```

以下命令用于查看是否安装成功:

```shell
sudo systemctl status ssh
```

（正确安装可以看到active）

使用防火墙（即 ufw）打开 SSH 22/TCP 端口

```shell
sudo systemctl status ssh
```

检查端口22是否正确打开

```shell
sudo ufw status
```

(正确打开可以看到ALLOW)

##### 2、安装和配置Docker(Linux Ubuntu)

安装Docker

```shell
sudo apt update
sudo apt install docker.io
```

将用户添加到docker组

```shell
sudo usermod -aG docker $USER
```

验证Docker安装

```
docker --version
```

(正确安装后可以看到Docker的版本)

拉取并运行"hello-world"docker镜像以验证docker可以正常运行

```
docker run hello-world
```

#### 2、准备客户端工作站

##### 1、安装Docker

由于结点使用相同的docker镜像可以更好的使用Ray进行工作，因此最好在客户机上也安装Docker。

安装Docker软件:

```shell
sudo apt update
sudo apt install docker.io
```

将用户添加到docker群组

```shell
sudo usermod -aG docker $USER
```

验证Docker的安装

```shell
docker --version
```

(正确安装可以看到Docker的版本)

##### 2、运行Ray Docker镜像

拉取Ray docker镜像:

```shell
docker pull rayproject/ray-ml:latest-cpu
```
这里没有使用支持GPU的镜像，因为这些虚拟机都没有GPU，如果你的服务器支持GPU，你可以改用`rayproject/ray-ml:latest-gpu`。

运行Ray docker容器：

```shell
docker run -i -t rayproject/ray-ml
```

在docker容器中，我们需要获取python和ray的版本，这样让客户端使用同样的版本才能正常运行。
获取python版本：

```shell
python --version
```

获取Ray python包版本：

首先输入如下命令进入python 解释器:

```shell
python
```

然后在python解释器中键入以下python脚本：

```python
import ray
print(ray.__version__)
```

此时输出ray的版本，然后退出python解释器，输入命令:

```python
quit()
```

最后退出docker容器:

```shell
exit
```

##### 3、设置SSH客户端

要部署 Ray 集群，需要使用 ssh-keygen 为 SSH 创建新的身份验证密钥对，以自动登录、单点登录和验证主机。创建密钥后，需要使用 ssh-copy-id 将公钥复制到 Ray 集群中的每个节点。

为SSH创建新的身份验证密钥对：

```shell
ssh-keygen
```

使用`ssh-copy-id`将公钥复制到每个节点。以一个头结点和三个工作节点为例，命令如下：

示例：
```shell
ssh-copy-id -i ~ /.ssh/i d_rsa. pub ssh_user@172.31.223.167
ssh-copy-id -i ~ /.ssh/i d_rsa. pub ssh_user@172.31.176.122
ssh-copy-id -i ~ /.ssh/i d_rsa. pub ssh_user@172.31.132.69
ssh-copy-id -i ~ /.ssh/i d_rsa. pub ssh_user@172.31.218.156
```
其中ssh_user是之后用SSH连接到这些节点时将会使用的用户名。请注意对工作节点和头结点使用的用户名都是相同的，你需要手动创建这些用户。每条命令的最后需要使用工作节点的实际IP地址。可以使用`hostname -I`来查看本机的IP。



##### 4、安装和配置Miniconda

这里将安装和配置**Miniconda**，这样客户端就能方便地使用与节点的docker镜像中相同的Python版本和ray版本。

Miniconda下载地址：[Miniconda下载]([https://www.anaconda.com/products/distribution#linux](https://docs.conda.io/en/latest/miniconda.html))

安装Miniconda：

```shell
bash ~/Downloads/Miniconda3-py310_22.11.1-1-Linux-x86_64.sh
```

安装完成之后，需要为Ray创建一个conda环境:

```shell
conda create --name ray python=3.7.13
```

接下来激活"Ray"环境:

```shell
conda activate ray
```

安装Ray python包:

```shell
pip install "ray[default]"==2.1.0
```

#### 3、使用Ray Up部署Ray集群

##### 1、下载YAML文件

```shell
wget https://raw.githubusercontent.com/ray-project/ray/master/python/ray/autoscaler/local/example-full.yaml _ _ _
```

##### 2、修改并重命名example-full.yaml文件

修改YAML文件的以下部分(以红色表示)：

```yaml
provider:
    type: local
    head_ip: YOUR_HEAD_NODE_HOSTNAME
    # You may need to supply a public ip for the head node if you need
    # to run `ray up` from outside of the Ray cluster's network
    # (e.g. the cluster is in an AWS VPC and you're starting ray from your laptop)
    # This is useful when debugging the local node provider with cloud VMs.
    # external_head_ip: YOUR_HEAD_PUBLIC_IP
    worker_ips: [WORKER_NODE_1_HOSTNAME, WORKER_NODE_2_HOSTNAME, ... ]
    # Optional when running automatic cluster management on prem. If you use a coordinator server,
    # then you can launch multiple autoscaling clusters on the same set of machines, and the coordinator
    # will assign individual nodes to clusters as needed.
    #    coordinator_address: "<host>:<port>"
```

- head_ip：将成为头节点的计算机的 IP 地址（例如，172.31.223.167）。

- worker_ips：将成为工作节点的计算机的 IP 地址（例如，[172.31.176.122，172.31.132.69，172.31.218.156]）。

修改YAML文件的以下部分（以红色表示）：

```YAML
# How Ray will authenticate with newly launched nodes.
auth:
    ssh_user: YOUR_USERNAME
    # You can comment out `ssh_private_key` if the following machines don't need a private key for SSH access to the Ray
    # cluster:
    #   (1) The machine on which `ray up` is executed.
    #   (2) The head node of the Ray cluster.
    #
    # The machine that runs ray up executes SSH commands to set up the Ray head node. The Ray head node subsequently
    # executes SSH commands to set up the Ray worker nodes. When you run ray up, ssh credentials sitting on the ray up
    # machine are copied to the head node -- internally, the ssh key is added to the list of file mounts to rsync to head node.
    # ssh_private_key: ~/.ssh/id_rsa
```

- ssh_user：将用于登录头节点计算机和工作节点计算机的用户名（例如，ray_client）。

修改YAML文件的以下部分（以红色表示）：

```yaml
# The minimum number of workers nodes to launch in addition to the head
# node. This number should be >= 0.
# Typically, min_workers == max_workers == len(worker_ips).
# This field is optional.
min_workers: TYPICALLY_THE_NUMBER_OF_WORKER_IPS

# The maximum number of workers nodes to launch in addition to the head node.
# This takes precedence over min_workers.
# Typically, min_workers == max_workers == len(worker_ips).
# This field is optional.
max_workers: TYPICALLY_THE_NUMBER_OF_WORKER_IPS
# The default behavior for manually managed clusters is
# min_workers == max_workers == len(worker_ips),
# meaning that Ray is started on all available nodes of the cluster.
# For automatically managed clusters, max_workers is required and min_workers defaults to 0.
```

- min_workers：worker_ips中IP地址的个数或3个。
- max_workers：worker_ips中IP地址的个数或3个。

保存更改并将文件重命名为“default-full.yaml”，因为集群名称是“default”。

##### 3、激活Ray Conda环境

运行以下命令以激活conda环境：

```shell
conda activate ray
```

##### 4、运行"ray up"

最后使用如下命令来部署Ray集群：

```shell
ray up default-full.yaml
```

使用如下命令来检查Ray集群的状态：

```shell
ray exec default-full.yaml 'ray status'
```

使用如下命令启动dashboard：

```shell
ray dashboard default-full.yaml
```

在浏览器中输入以下地址以打开**Ray Dashboard**

```shell
http://127.0.0.1:8265
```

运行一个microbenchmark来确保Ray集群按预期的工作：

```shell
ray exec default-full.yaml 'ray microbenchmark'
```

#### 4、向ray集群提交任务

为了方便编程，建议将VScode连接到本地的docker容器“rayproject/ray-ml”，这样你就可以看到该容器中安装的所有的包。

假设你写好的python文件叫做my_script.py，那么可以使用如下命令将它提交到ray集群并运行：
```shell
RAY_ADDRESS='http://127.0.0.1:8265' ray job submit --working-dir . -- python my_script.py
```
需要确保my_script.py在当前工作目录下。

这里进行测试时使用的命令如下：

```bash
Desktop目录下

RAY_ADDRESS='http://127.0.0.1:8265' ray job submit --working-dir . -- python markov.py 1000 10000

RAY_ADDRESS='http://127.0.0.1:8265' ray job submit --working-dir . -- python markov.py 100 10000

RAY_ADDRESS='http://127.0.0.1:8265' ray job submit --working-dir . -- python markov.py 10 10000

RAY_ADDRESS='http://127.0.0.1:8265' ray job submit --working-dir . -- python markov.py 1 10000
```

> **注意**这里由于**vlab虚拟机硬件资源的限制**，没有办法做到和其他情形下的运算规模相同的地步，事实上，在规模达到10000000时就必然会报出内存不足的错误而后自动终止程序。所以这里只能够使用很小的参数。
>
> 另外，由于虚拟机似乎不能够很好的支持dashboard，所以经过多次失败后最终是使用命令行界面进行部署的，也尝试了ssh终端登陆多开，在运行时用命令查看状态，不过还是达不到dashboard的效果。所以最终我们只选用任务规模、拆分数、运行时间（程序输出）作为主要关注对象。

使用`ray down`来关闭Ray集群：

```shell
ray down default-full.yaml
```

需要注意的是，关闭了Ray集群并没有关闭Ray容器，需要手动关闭Ray容器。



**Ray docker部署的运行结果（命令如上）**

![pic9](src/pic9.jpg)

![pic8](src/pic8.jpg)

![pic10](src/pic10.jpg)

![pic11](src/pic11.jpg)

### Summary

**total: 10000**

| pc_num  | 1000 | 100  | 10   | 1    |
| ------- | ---- | ---- | ---- | ---- |
| time(s) | 4.48 | 2.56 | 2.61 | 2.36 |

这里的分布趋势仍然与此前的解释相符。



#### 5、分布式部署的性能测试

**多机配置**

三台虚拟机，USTC Vlab的标准配置。



**Dashboard**

![pic5](src/pic5.jpg)



#### 6、分布式部署的性能分析、优化

初始pc_num=100000，单位秒(s)，pc_num为任务拆分数

| 任务分配数：pc_num     | pc_num=100000            | pc_num=100        | pc_num = 30       | pc_num=12         |
| ---------------------- | ------------------------ | ----------------- | ----------------- | ----------------- |
| 无Ray                  | 45.20691394805908(+/-2s) | 同                | 同                | 同                |
| 单机版ray              | 185.89336466789246       | 45.6342716217041  | 48.51240634918213 | 46.83416128158569 |
| 多机ray集群（3个结点） | 171.68697834014893       | 50.97406363487244 | 43.56711173057556 | 49.56426119804382 |

任务规模在100000时，多机部署相对于单机部署有百分之十五的性能提升。

这里单机版我们得到了与此前在另一台计算机上运行相似的结果，单机调参优化最高达到`307%`  。

多机版本不同的是，在任务柴分数进一步减小，从30减少到12时性能出现了百分之四十的回降，这是由于计算资源没有得到充分利用，这一点抵消了进程间通讯的开销。可以想见，如果任务规模大幅扩大，则多机的性能会较大幅度的优于单机。多机调参优化最高达到`294%`。



### 发布链接

CSDN：

https://blog.csdn.net/m0_62162986/article/details/131316894