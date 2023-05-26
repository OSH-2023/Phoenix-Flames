#include <stdint.h>
#include "string.h"

#include "console.h"
#include "syscall.h"

extern int start_bss;
extern int end_bss;

void clear_bss() {
  uint8_t* start_ptr = (uint8_t*)&start_bss;
  uint8_t* end_ptr = (uint8_t*)&end_bss;

  for (; start_ptr < end_ptr; start_ptr++) {
    *start_ptr = 0;
  }
}

int user_main();

void start() __attribute__((used));
void __attribute__((section(".text.entry"))) start() {
  clear_bss();
  sys_exit(user_main());
}

int user_main() {
  int a = 12;
  int b = 11;
  printf("a=%d,b=%d,a+b=%d\n", a, b, a + b);
}
