//to be done

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


 pub fn lookupCap(thread:&tcb_t,  cPtr:cptr_t)->lookupCap_ret_t
{
    let lu_ret:lookupSlot_raw_ret_t;
    let ret:lookupCap_ret_t;

    lu_ret = lookupSlot(thread, cPtr);
    if (unlikely(lu_ret.status != EXCEPTION_NONE)) {
        ret.status = lu_ret.status;
        ret.cap = cap_null_cap_new();
        return ret;
    }

    ret.status = EXCEPTION_NONE;
    ret.cap = lu_ret.slot->cap;
    return ret;
}


// lookupCapAndSlot_ret_t lookupCapAndSlot(tcb_t *thread, cptr_t cPtr)
// {
//     lookupSlot_raw_ret_t lu_ret;
//     lookupCapAndSlot_ret_t ret;

//     lu_ret = lookupSlot(thread, cPtr);
//     if (unlikely(lu_ret.status != EXCEPTION_NONE)) {
//         ret.status = lu_ret.status;
//         ret.slot = NULL;
//         ret.cap = cap_null_cap_new();
//         return ret;
//     }

//     ret.status = EXCEPTION_NONE;
//     ret.slot = lu_ret.slot;
//     ret.cap = lu_ret.slot->cap;
//     return ret;
// }

pub fn lookupSlot(thread:&tcb_t, capptr:cptr_t )->lookupSlot_raw_ret_t
{
    let threadRoot:cap_t ;
    let res_ret:resolveAddressBits_ret_t;
    let ret:lookupSlot_raw_ret_t;

    threadRoot = TCB_PTR_CTE_PTR(thread, tcbCTable)->cap;
    res_ret = resolveAddressBits(threadRoot, capptr, wordBits);

    ret.status = res_ret.status;
    ret.slot = res_ret.slot;
    return ret;
}

// lookupSlot_ret_t lookupSlotForCNodeOp(bool_t isSource, cap_t root, cptr_t capptr,
//                                       word_t depth)
// {
//     resolveAddressBits_ret_t res_ret;
//     lookupSlot_ret_t ret;

//     ret.slot = NULL;

//     if (unlikely(cap_get_capType(root) != cap_cnode_cap)) {
//         current_syscall_error.type = seL4_FailedLookup;
//         current_syscall_error.failedLookupWasSource = isSource;
//         current_lookup_fault = lookup_fault_invalid_root_new();
//         ret.status = EXCEPTION_SYSCALL_ERROR;
//         return ret;
//     }

//     if (unlikely(depth < 1 || depth > wordBits)) {
//         current_syscall_error.type = seL4_RangeError;
//         current_syscall_error.rangeErrorMin = 1;
//         current_syscall_error.rangeErrorMax = wordBits;
//         ret.status = EXCEPTION_SYSCALL_ERROR;
//         return ret;
//     }
//     res_ret = resolveAddressBits(root, capptr, depth);
//     if (unlikely(res_ret.status != EXCEPTION_NONE)) {
//         current_syscall_error.type = seL4_FailedLookup;
//         current_syscall_error.failedLookupWasSource = isSource;
//         /* current_lookup_fault will have been set by resolveAddressBits */
//         ret.status = EXCEPTION_SYSCALL_ERROR;
//         return ret;
//     }

//     if (unlikely(res_ret.bitsRemaining != 0)) {
//         current_syscall_error.type = seL4_FailedLookup;
//         current_syscall_error.failedLookupWasSource = isSource;
//         current_lookup_fault =
//             lookup_fault_depth_mismatch_new(0, res_ret.bitsRemaining);
//         ret.status = EXCEPTION_SYSCALL_ERROR;
//         return ret;
//     }

//     ret.slot = res_ret.slot;
//     ret.status = EXCEPTION_NONE;
//     return ret;
// }

// lookupSlot_ret_t lookupSourceSlot(cap_t root, cptr_t capptr, word_t depth)
// {
//     return lookupSlotForCNodeOp(true, root, capptr, depth);
// }

// lookupSlot_ret_t lookupTargetSlot(cap_t root, cptr_t capptr, word_t depth)
// {
//     return lookupSlotForCNodeOp(false, root, capptr, depth);
// }

// lookupSlot_ret_t lookupPivotSlot(cap_t root, cptr_t capptr, word_t depth)
// {
//     return lookupSlotForCNodeOp(true, root, capptr, depth);
// }

// resolveAddressBits_ret_t resolveAddressBits(cap_t nodeCap, cptr_t capptr, word_t n_bits)
// {
//     resolveAddressBits_ret_t ret;
//     word_t radixBits, guardBits, levelBits, guard;
//     word_t capGuard, offset;
//     cte_t *slot;

//     ret.bitsRemaining = n_bits;
//     ret.slot = NULL;

//     if (unlikely(cap_get_capType(nodeCap) != cap_cnode_cap)) {
//         current_lookup_fault = lookup_fault_invalid_root_new();
//         ret.status = EXCEPTION_LOOKUP_FAULT;
//         return ret;
//     }

//     while (1) {
//         radixBits = cap_cnode_cap_get_capCNodeRadix(nodeCap);
//         guardBits = cap_cnode_cap_get_capCNodeGuardSize(nodeCap);
//         levelBits = radixBits + guardBits;

//         /* Haskell error: "All CNodes must resolve bits" */
//         assert(levelBits != 0);

//         capGuard = cap_cnode_cap_get_capCNodeGuard(nodeCap);

//         /* The MASK(wordRadix) here is to avoid the case where
//          * n_bits = wordBits (=2^wordRadix) and guardBits = 0, as it violates
//          * the C spec to shift right by more than wordBits-1.
//          */
//         guard = (capptr >> ((n_bits - guardBits) & MASK(wordRadix))) & MASK(guardBits);
//         if (unlikely(guardBits > n_bits || guard != capGuard)) {
//             current_lookup_fault =
//                 lookup_fault_guard_mismatch_new(capGuard, n_bits, guardBits);
//             ret.status = EXCEPTION_LOOKUP_FAULT;
//             return ret;
//         }

//         if (unlikely(levelBits > n_bits)) {
//             current_lookup_fault =
//                 lookup_fault_depth_mismatch_new(levelBits, n_bits);
//             ret.status = EXCEPTION_LOOKUP_FAULT;
//             return ret;
//         }

//         offset = (capptr >> (n_bits - levelBits)) & MASK(radixBits);
//         slot = CTE_PTR(cap_cnode_cap_get_capCNodePtr(nodeCap)) + offset;

//         if (likely(n_bits == levelBits)) {
//             ret.status = EXCEPTION_NONE;
//             ret.slot = slot;
//             ret.bitsRemaining = 0;
//             return ret;
//         }

//         /** GHOSTUPD: "(\<acute>levelBits > 0, id)" */

//         n_bits -= levelBits;
//         nodeCap = slot->cap;

//         if (unlikely(cap_get_capType(nodeCap) != cap_cnode_cap)) {
//             ret.status = EXCEPTION_NONE;
//             ret.slot = slot;
//             ret.bitsRemaining = n_bits;
//             return ret;
//         }
//     }

//     UNREACHABLE();
// }

