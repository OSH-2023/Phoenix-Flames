#include "syscall.h"

#include "string.h"

#define SYSCALL_WRITE 64
#define SYSCALL_EXIT 93

int64_t syscall(uint64_t id, uint64_t arg0, uint64_t arg1, uint64_t arg2) {
  int64_t ret;

  __asm__ volatile(
      "mv a0,%1\n"
      "mv a1,%2\n"
      "mv a2,%3\n"
      "mv a7,%4\n"
      "ecall\n"
      "mv %0,a0"
      : "=r"(ret)
      : "r"(arg0), "r"(arg1), "r"(arg2), "r"(id)
      : "a0", "a1", "a2", "a7"  // Clobbered registers
  );

  return ret;
}

void sys_exit(int exit_code) {
  syscall(SYSCALL_EXIT, (uint64_t)exit_code, 0, 0);
  // exit fail:
  char message[] = "Error: the program should exit\n";
  sys_write(STDOUT, message, strlen(message));
  while (1)
    ;
}

uint64_t sys_write(int fd, const char* buf, size_t len) {
  return syscall(SYSCALL_WRITE, (uint64_t)fd, (uint64_t)buf, len);
}