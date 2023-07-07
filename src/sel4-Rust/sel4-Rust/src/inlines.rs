use crate::failures::syscall_error_t;
use crate::object::*;

extern "C" {
    pub static mut current_lookup_fault: lookup_fault_t;
    pub static mut current_fault: seL4_Fault_t;
    pub static mut current_syscall_error: syscall_error_t;
}
