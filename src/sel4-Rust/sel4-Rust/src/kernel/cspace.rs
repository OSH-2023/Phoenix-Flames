// jwh:7.5 18:00
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
use core::intrinsics::{likely,unlikely};

use super::thread::*;
use crate::failures::*;
use crate::machine::*;
use crate::model::statedata::*;
use crate::object::*;
use crate::types::*;

//from cspace.h
#[derive(Clone)]
pub struct lookupCap_ret {
    pub status: exception_t,
    pub cap: cap_t,
}
pub type lookupCap_ret_t = lookupCap_ret;

#[derive(Clone)]
pub struct lookupCapAndSlot_ret<'a> {
    pub status: exception_t,
    pub cap: cap_t,
    pub slot: &'a cte_t,
}
pub type lookupCapAndSlot_ret_t<'a> = lookupCapAndSlot_ret<'a>;

#[derive(Clone)]
pub struct lookupSlot_raw_ret<'a> {
    pub status: exception_t,
    pub slot: &'a cte_t,
}
pub type lookupSlot_raw_ret_t<'a> = lookupSlot_raw_ret<'a>;

#[derive(Clone)]
pub struct lookupSlot_ret<'a> {
    pub status: exception_t,
    pub slot: &'a cte_t,
}
pub type lookupSlot_ret_t<'a> = lookupSlot_ret<'a>;

#[derive(Clone)]
pub struct resolveAddressBits_ret<'a> {
    pub status: exception_t,
    pub slot: &'a cte_t,
    pub bitsRemaining: u64,
}
pub type resolveAddressBits_ret_t<'a> = resolveAddressBits_ret<'a>;


pub fn lookupCap(thread: *mut tcb_t, cPtr: u64) -> lookupCap_ret_t {
    let mut lu_ret = lookupSlot(thread, cPtr);
    if lu_ret.status != 0u64 {
        return lookupCap_ret_t {
            status: lu_ret.status,
            cap: cap_null_cap_new(),
        };
    }
    lookupCap_ret_t {
        status: 0u64,
        cap: (*lu_ret.slot).cap,
    }
}


pub fn lookupCapAndSlot(thread: *mut  tcb_t,cPtr:u64)->lookupCapAndSlot_ret_t
{
    let mut lu_ret=lookupSlot(thread,cPtr);
    if lu_ret.status != 0u64 {
        return lookupCapAndSlot_ret_t{
            status:lu_ret.status,
            slot:std::ptr::null_mut(),
            cap:cap_null_cap_new(),
        }
    }
    lookupCapAndSlot_ret_t {
        status: 0u64,
        cap: (*lu_ret.slot).cap,
        slot: lu_ret.slot,
    }
}


pub fn lookupSlot(thread: *mut tcb_t, capptr:u64 )->lookupSlot_raw_ret_t
{
    let threadRoot = (*tcb_ptr_cte_ptr(thread, tcb_cnode_index::tcbCTable as u64)).cap;
    let res_ret = resolveAddressBits(threadRoot, capptr, wordBits);
    lookupSlot_raw_ret_t {
        status: res_ret.status,
        slot: res_ret.slot,
    }
}

pub fn lookupSlotForCNodeOp(isSource:bool_t,root:cap_t,capptr:u64,depth:u64)->lookupSlot_ret_t
{
    let res_ret:resolveAddressBits_ret_t ;
    let ret:lookupSlot_ret_t;
    
    ret.slot=std::ptr::null_mut();
    if cap_get_capType(root) != cap_tag_t::cap_cnode_cap as u64 {
        current_syscall_error.type_ = seL4_Error::seL4_FailedLookup as u64;
        current_syscall_error.failedLookupWasSource = isSource;
        current_lookup_fault = lookup_fault_invalid_root_new();
        return lookupSlot_ret_t {
            status: exception::EXCEPTION_SYSCALL_ERROR as u64,
            slot: std::ptr::null_mut()
        };
    }
    if depth < 1 || depth > wordBits {
        current_syscall_error.type_ = seL4_Error::seL4_RangeError as u64;
        current_syscall_error.rangeErrorMin = 1;
        current_syscall_error.rangeErrorMax = wordBits;
        return lookupSlot_ret_t {
            status: exception::EXCEPTION_SYSCALL_ERROR as u64,
            slot: std::ptr::null_mut(),
        };
    }

    let res_ret = resolveAddressBits(root, capptr, depth);
    if res_ret.status != 0u64 {
        current_syscall_error.type_ = seL4_Error::seL4_FailedLookup as u64;
        current_syscall_error.failedLookupWasSource = isSource;
        return lookupSlot_ret_t {
            status: exception::EXCEPTION_SYSCALL_ERROR as u64,
            slot: exception::EXCEPTION_SYSCALL_ERROR as u64,
        };
    }
    if res_ret.bitsRemaining != 0 {
        current_syscall_error.type_ = seL4_Error::seL4_FailedLookup as u64;
        current_syscall_error.failedLookupWasSource = isSource;
        current_lookup_fault = lookup_fault_depth_mismatch_new(0, res_ret.bitsRemaining);
        return lookupSlot_ret_t {
            status: exception::EXCEPTION_SYSCALL_ERROR as u64,
            slot: exception::EXCEPTION_SYSCALL_ERROR as u64,
        };
    }
    lookupSlot_ret_t {
        status: 0u64,
        slot: res_ret.slot,
    }
}
pub  fn lookupSourceSlot(
    root: cap_t,
    capptr: u64,
    depth: u64,
) -> lookupSlot_ret_t {
    lookupSlotForCNodeOp(1u64, root, capptr, depth)
}


pub fn lookupTargetSlot(
    root: cap_t,
    capptr: u64,
    depth: u64,
) -> lookupSlot_ret_t {
    lookupSlotForCNodeOp(0u64, root, capptr, depth)
}

pub fn lookupPivotSlot(root: cap_t, capptr: u64, depth: u64) -> lookupSlot_ret_t {
    lookupSlotForCNodeOp(1u64, root, capptr, depth)
}

macro_rules! MASK {
    ($x:expr) => {
        (1u64 << ($x)) - 1u64
    };
}


pub fn resolveAddressBits(
    mut nodeCap: cap_t,
    capptr: u64,
    mut n_bits: u64,
) -> resolveAddressBits_ret_t {
    let mut ret = resolveAddressBits_ret_t {
        status: 0u64,
        slot:std::ptr::null_mut(),
        bitsRemaining: n_bits,
    };
    if cap_get_capType(nodeCap) != cap_tag_t::cap_cnode_cap as u64 {
        current_lookup_fault = lookup_fault_invalid_root_new();
        ret.status = exception::EXCEPTION_LOOKUP_FAULT as u64;
        return ret;
    }

    loop {
        let radixBits = cap_cnode_cap_get_capCNodeRadix(nodeCap);
        let guardBits = cap_cnode_cap_get_capCNodeGuardSize(nodeCap);
        let levelBits = radixBits + guardBits;
        let capGuard = cap_cnode_cap_get_capCNodeGuard(nodeCap);
        let guard: u64 = (capptr >> ((n_bits - guardBits) & MASK!(wordRadix))) & MASK!(guardBits);
        if guardBits > n_bits || guard != capGuard {
            current_lookup_fault = lookup_fault_guard_mismatch_new(capGuard, n_bits, guardBits);
            ret.status = exception::EXCEPTION_LOOKUP_FAULT as u64;
            return ret;
        }
        if levelBits > n_bits {
            current_lookup_fault = lookup_fault_depth_mismatch_new(levelBits, n_bits);
            ret.status = exception::EXCEPTION_LOOKUP_FAULT as u64;
            return ret;
        }
        let offset: u64 = (capptr >> (n_bits - levelBits)) & MASK!(radixBits);
        let slot = CTE_PTR(cap_cnode_cap_get_capCNodePtr(nodeCap)) + offset;
        if n_bits <= levelBits {
            ret.status = 0u64;
            ret.slot = slot;
            ret.bitsRemaining = 0;
            return ret;
        }
        n_bits -= levelBits;
        nodeCap = (*slot).cap;
        if cap_get_capType(nodeCap) != cap_tag_t::cap_cnode_cap as u64 {
            ret.status = exception::EXCEPTION_NONE as u64;
            ret.slot = slot;
            ret.bitsRemaining = n_bits;
            return ret;
        }
    }
    ret.status = 0u64;
    ret
}