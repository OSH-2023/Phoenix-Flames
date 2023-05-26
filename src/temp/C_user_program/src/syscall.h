#pragma once
#include <stddef.h>
#include <stdint.h>
#define STDOUT 1
void sys_exit(int code);

uint64_t sys_write(int fd,const char* buf,size_t len);