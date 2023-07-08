// jwh:7.5 18:00
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
use core::intrinsics::{likely, unlikely};


use crate::failures::*;
use crate::inlines::*;
use crate::machine::*;
use crate::model::statedata::*;
use crate::object::*;
use crate::types::*;
use crate::CTE_PTR;
use crate::MASK;

//from cspace.h
#[derive(Clone, Copy)]
#[repr(C)]
pub struct lookupCap_ret {
    pub status: exception_t,
    pub cap: cap_t,
}
pub type lookupCap_ret_t = lookupCap_ret;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct lookupCapAndSlot_ret {
    pub status: exception_t,
    pub cap: cap_t,
    pub slot: *mut cte_t,
}
pub type lookupCapAndSlot_ret_t = lookupCapAndSlot_ret;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct lookupSlot_raw_ret {
    pub status: exception_t,
    pub slot: *mut cte_t,
}
pub type lookupSlot_raw_ret_t = lookupSlot_raw_ret;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct lookupSlot_ret {
    pub status: exception_t,
    pub slot: *mut cte_t,
}
pub type lookupSlot_ret_t = lookupSlot_ret;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct resolveAddressBits_ret {
    pub status: exception_t,
    pub slot: *mut cte_t,
    pub bitsRemaining: u64,
}
pub type resolveAddressBits_ret_t = resolveAddressBits_ret;

// #[no_mangle]
// pub extern "C" fn lookupCap(thread: *mut tcb_t, cPtr: u64) -> lookupCap_ret_t {
//     let lu_ret = lookupSlot(thread, cPtr);
//     if lu_ret.status != 0u64 {
//         return lookupCap_ret_t {
//             status: lu_ret.status,
//             cap: cap_null_cap_new(),
//         };
//     }
//     unsafe {
//         lookupCap_ret_t {
//             status: 0u64,
//             cap: (*lu_ret.slot).cap,
//         }
//     }
// }

// #[no_mangle]
// pub extern "C" fn lookupCapAndSlot(thread: *mut tcb_t, cPtr: u64) -> lookupCapAndSlot_ret_t {
//     let lu_ret = lookupSlot(thread, cPtr);
//     if lu_ret.status != 0u64 {
//         return lookupCapAndSlot_ret_t {
//             status: lu_ret.status,
//             slot: 0 as *mut cte,
//             cap: cap_null_cap_new(),
//         };
//     }
//     unsafe {
//         lookupCapAndSlot_ret_t {
//             status: 0u64,
//             cap: (*lu_ret.slot).cap,
//             slot: lu_ret.slot,
//         }
//     }
// }

// #[no_mangle]
// pub extern "C" fn lookupSlot(thread: *mut tcb_t, capptr: u64) -> lookupSlot_raw_ret_t {
//     let threadRoot: cap;
//     unsafe {
//         threadRoot = (*TCB_PTR_CTE_PTR(thread, 0)).cap;
//     }
//     let res_ret = resolveAddressBits(threadRoot, capptr, wordBits);
//     lookupSlot_raw_ret_t {
//         status: res_ret.status,
//         slot: res_ret.slot,
//     }
// }

// #[no_mangle]
// pub extern "C" fn lookupSlotForCNodeOp(
//     isSource: bool_t,
//     root: cap_t,
//     capptr: u64,
//     depth: u64,
// ) -> lookupSlot_ret_t {
//     let mut ret= lookupSlot_ret_t{
//         status:0,
//         slot:0 as *mut cte
//     };

//     ret.slot = 0 as *mut cte;
//     if cap_get_capType(root) != 10 as u64 {
//         unsafe {
//             current_syscall_error.error_type = 6 as u64;
//             current_syscall_error.failedLookupWasSource = isSource;
//             current_lookup_fault = lookup_fault_invalid_root_new();
//         }
//         return lookupSlot_ret_t {
//             status: exception::EXCEPTION_SYSCALL_ERROR as u64,
//             slot: 0 as *mut cte,
//         };
//     }
//     if depth < 1 || depth > wordBits {
//         unsafe {
//             current_syscall_error.error_type = seL4_Error::seL4_RangeError as u64;
//             current_syscall_error.rangeErrorMin = 1;
//             current_syscall_error.rangeErrorMax = wordBits;
//         }
//         return lookupSlot_ret_t {
//             status: exception::EXCEPTION_SYSCALL_ERROR as u64,
//             slot: 0 as *mut cte,
//         };
//     }

//     let res_ret = resolveAddressBits(root, capptr, depth);
//     if res_ret.status != 0u64 {
//         unsafe {
//             current_syscall_error.error_type = seL4_Error::seL4_FailedLookup as u64;
//             current_syscall_error.failedLookupWasSource = isSource;
//         }
//         /* current_lookup_fault will have been set by resolveAddressBits */
//         ret.status = exception::EXCEPTION_SYSCALL_ERROR as u64;
//         return ret;
//     }
//     if res_ret.bitsRemaining != 0 {
//         unsafe {
//             current_syscall_error.error_type = seL4_Error::seL4_FailedLookup as u64;
//             current_syscall_error.failedLookupWasSource = isSource;
//             current_lookup_fault = lookup_fault_depth_mismatch_new(0, res_ret.bitsRemaining);
//         }
//         ret.status = exception::EXCEPTION_SYSCALL_ERROR as u64;
//         return ret;
//     }

//     ret.slot = res_ret.slot;
//     ret.status = exception::EXCEPTION_NONE as u64;
//     ret
// }

// #[no_mangle]
// pub extern "C" fn lookupSourceSlot(root: cap_t, capptr: u64, depth: u64) -> lookupSlot_ret_t {
//     lookupSlotForCNodeOp(1u64, root, capptr, depth)
// }

// #[no_mangle]
// pub extern "C" fn lookupTargetSlot(root: cap_t, capptr: u64, depth: u64) -> lookupSlot_ret_t {
//     lookupSlotForCNodeOp(0u64, root, capptr, depth)
// }

// #[no_mangle]
// pub extern "C" fn lookupPivotSlot(root: cap_t, capptr: u64, depth: u64) -> lookupSlot_ret_t {
//     lookupSlotForCNodeOp(1u64, root, capptr, depth)
// }

#[no_mangle]
pub extern "C" fn resolveAddressBits(
    mut nodeCap: cap_t,
    capptr: u64,
    mut n_bits: u64,
) -> resolveAddressBits_ret_t {
    let mut ret = resolveAddressBits_ret_t {
        status: 0u64,
        slot: 0 as *mut cte,
        bitsRemaining: n_bits,
    };
    if cap_get_capType(nodeCap) != cap_tag_t::cap_cnode_cap as u64 {
        unsafe {
            current_lookup_fault = lookup_fault_invalid_root_new();
        }
        ret.status = exception::EXCEPTION_LOOKUP_FAULT as u64;
        return ret;
    }

    loop {
        let radixBits = cap_cnode_cap_get_capCNodeRadix(nodeCap);
        let guardBits = cap_cnode_cap_get_capCNodeGuardSize(nodeCap);
        let levelBits = radixBits + guardBits;
        
        /* Haskell error: "All CNodes must resolve bits" */
        assert!(levelBits != 0);

        let capGuard = cap_cnode_cap_get_capCNodeGuard(nodeCap);
        let guard: u64 = (capptr >> ((n_bits - guardBits) & MASK!(wordRadix))) & MASK!(guardBits);
        
        if guardBits > n_bits || guard != capGuard {
            unsafe {
                current_lookup_fault = lookup_fault_guard_mismatch_new(capGuard, n_bits, guardBits);
            }
            ret.status = exception::EXCEPTION_LOOKUP_FAULT as u64;
            return ret;
        }
        if levelBits > n_bits {
            unsafe {
                current_lookup_fault = lookup_fault_depth_mismatch_new(levelBits, n_bits);
            }
            ret.status = exception::EXCEPTION_LOOKUP_FAULT as u64;
            return ret;
        }
        let offset: u64 = (capptr >> (n_bits - levelBits)) & MASK!(radixBits);

        let slot = (cap_cnode_cap_get_capCNodePtr(nodeCap) + offset) as *mut cte;
        if n_bits == levelBits {
            ret.status = exception::EXCEPTION_NONE as u64;
            ret.slot = slot;
            ret.bitsRemaining = 0;
            return ret;
        }

        n_bits -= levelBits;
        unsafe{
            nodeCap = (*slot).cap;
        }

        if cap_get_capType(nodeCap) != cap_tag_t::cap_cnode_cap as u64 {
            ret.status = exception::EXCEPTION_NONE as u64;
            ret.slot = slot;
            ret.bitsRemaining = n_bits;
            return ret;
        }
    }

    panic!("Unreachable!");
}
