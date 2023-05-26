#include "console.h"

#include <stdarg.h>
#include "string.h"

#include "syscall.h"

#define INT_STR_BUF_LEN 11

void print_int(int d);

void printf(const char* format, ...) {
  va_list args;
  va_start(args, format);

  char c;
  int d;
  char* s;

  while (*format) {
    if (*format == '%') {
      format++;
      switch (*format) {
        case 'c':
          c = (char)va_arg(args, int);
          sys_write(STDOUT, &c, 1);
          break;
        case 'd':
          d = va_arg(args, int);
          print_int(d);
          break;
        case 's':
          s = va_arg(args, char*);
          sys_write(STDOUT, s, strlen(s));
          break;
        default:
          // Handle unsupported format specifier
          char error_message[] = "Error: unsupported format specifier\n";
          sys_write(STDOUT, error_message, strlen(error_message));
          sys_exit(1);
          break;
      }
    } else {
      sys_write(STDOUT, format, 1);
    }
    format++;
  }

  va_end(args);
}

void tostring(char str[], int num) {
  if (num == 0) {
    str[0] = '0';
    str[1] = '\0';
    return;
  }

  int i, rem, len = 0, n, num_len;

  if (num > 0)
    n = num;
  else {
    len++;
    n = -num;
  }

  while (n != 0) {
    len++;
    n /= 10;
  }

  if (num < 0) {
    str[0] = '-';
    num_len = len - 1;
    num = -num;
  } else
    num_len = len;

  for (i = 0; i < num_len; i++) {
    rem = num % 10;
    num = num / 10;
    str[len - (i + 1)] = rem + '0';
  }
  str[len] = '\0';
}

void print_int(int d) {
  char buf[INT_STR_BUF_LEN];
  tostring(buf, d);
  sys_write(STDOUT, buf, strlen(buf));
}
