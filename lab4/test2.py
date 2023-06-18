import ray
import random
import time

ray.init()


def matrix_mul(matrix1, matrix2, size):
    matrix_res = [[0.0]*size for item in range(size)]
    for row in range(0, size):
        for col in range(0, size):
            matrix_res[row][col] = 0.0
            for k in range(0, size):
                matrix_res[row][col] += matrix1[row][k] * matrix2[k][col]
    return matrix_res


@ray.remote
class Worker(object):
    def __init__(self):
        self.size = 10
        self.matrix0 = self.matrix_init2()
        self.res = self.matrix0.copy()
        self.all_times = 10000

    def matrix_init(self):
        # 单纯的随机数矩阵
        matrix = [[0.0]*self.size for item in range(self.size)]
        for row in range(self.size):
            for col in range(self.size):
                matrix[row][col] = 2 * random.random() - 1.0
        print("Now we have a matrix\n", matrix)
        return matrix

    def matrix_init2(self):
        # 每行均为正数浮点随机数，单行和为1（归一化）
        matrix = [[0.0] * self.size for item in range(self.size)]
        for row in range(self.size):
            for col in range(self.size):
                matrix[row][col] = random.random()
            total = sum(matrix[row])
            for col in range(self.size):
                matrix[row][col] /= total
        print("Now we have a matrix\n", matrix)
        return matrix

    # 转移矩阵的极限分布
    # 计算若干个矩阵的相乘
    def calculate(self, times):
        print("I start doing my work.")
        cur_time = time.time()
        task_res = self.matrix0.copy()
        for k in range(times-1):
            task_res = matrix_mul(self.res, self.matrix0, self.size)
        print("I have finished my work, duration: ", time.time() - cur_time)
        return task_res


if __name__ == '__main__':
    cur_time=time.time()
    worker = Worker.remote()
    worker.all_times = 1000
    pc_num = 10
    temps=[]
    for i in range(pc_num):
        temp = worker.calculate.remote(worker.all_times // pc_num)
        temps.append(temp)

    result_list = ray.get(temps)

    result = result_list[0]
    for m in result_list:
        result = matrix_mul(result, m, 10)
    print("final matrix: \n", result)
    print("total duration: ", time.time() - cur_time)
