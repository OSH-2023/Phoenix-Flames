//to be done

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

/*
Including contents from:
api/failures.h
 */
pub use crate::libsel4::constants::*;
pub use crate::libsel4::errors::*;
use crate::types::basic_types::*;
use crate::object::*;

//from api/failures.h
#[derive(Clone,Copy)]
#[repr(C)]
pub enum exception {
    EXCEPTION_NONE,
    EXCEPTION_FAULT,
    EXCEPTION_LOOKUP_FAULT,
    EXCEPTION_SYSCALL_ERROR,
    EXCEPTION_PREEMPTED,
}
pub type exception_t = word_t;

pub type syscall_error_type_t = word_t;

#[derive(Clone,Copy)]
#[repr(C)]
pub struct syscall_error {
    pub invalidArgumentNumber: word_t,
    pub invalidCapNumber: word_t,
    pub rangeErrorMin: word_t,
    pub rangeErrorMax: word_t,
    pub memoryLeft: word_t,
    pub failedLookupWasSource: bool_t,
    pub error_type: syscall_error_type_t,
}
pub type syscall_error_t = syscall_error;

