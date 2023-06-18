import ray
import random
import time

ray.init()


def matrix_mul(matrix1, matrix2, size):
    matrix_res = [[0.0]*size for item in range(size)]
    for row in range(0, size):
        for col in range(0, size):
            matrix_res[row][col] = 0.0
            for k1 in range(0, size):
                for k2 in range(0, size):
                    matrix_res[row][col] += matrix1[row][k1] * matrix2[k2][col]
    return matrix_res


@ray.remote
class Worker(object):
    def __init__(self):
        self.size = 10
        self.matrix0 = self.matrix_init()
        self.res = self.matrix0.copy()
        self.all_times = 10000

    def matrix_init(self):
        matrix = [[0.0]*self.size for item in range(self.size)]
        for row in range(self.size):
            for col in range(self.size):
                matrix[row][col] = 2 * random.random() - 1.0
        print("Now we have a matrix\n", matrix)
        return matrix

    # 转移矩阵的极限分布
    # 计算若干个矩阵的相乘
    def calculate(self, times):
        print("I start doing my work.")
        cur_time = time.time()
        for k in range(times):
            self.res = matrix_mul(self.res, self.matrix0, self.size)
        print("I have finished my work, duration: ", time.time() - cur_time)
        return self.res


if __name__ == '__main__':
    worker = Worker.remote()
    worker.all_times = 1000
    pc_num = 10
    temps=[]
    for i in range(pc_num):
        temp = worker.calculate.remote(worker.all_times // pc_num)
        temps.append(temp)

    result_list = ray.get(temps)
    result = worker.matrix0
    for m in result_list:
        result = matrix_mul(result, m, worker.size)
    print("final matrix: \n", result)
