#include "string.h"

size_t strlen (const char *__s){
    size_t len=0;
    while(*__s){
        len++;
        __s++;
    }
    return len;
}