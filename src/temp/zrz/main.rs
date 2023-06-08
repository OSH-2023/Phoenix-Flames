#[allow(non_snake_case)]
#[allow(non_camel_case_types)]

mod seL4_data;

use seL4_data::data_type::*;

extern "C"{
    static mut current_lookup_fault: lookup_fault_t;
    static mut current_syscall_error: syscall_error_t;
}

#[no_mangle]
pub extern "C" fn resolveAddressBits(mut nodeCap:cap_t, capptr:cptr_t, mut n_bits:word_t) -> resolveAddressBits_ret_t {
    let mut ret:resolveAddressBits_ret_t = resolveAddressBits_ret { status: 0u64, slot: 0u64 as *mut cte_t, bitsRemaining: 0 };
    let radixBits:word_t;
    let guardBits:word_t;
    let levelBits:word_t;
    let guard:word_t;
    let capGuard:word_t;
    let offset:word_t;
    let slot:*mut cte_t;

    ret.bitsRemaining = n_bits;
    while true {
        let radixBits = cap_cnode_cap_get_capCNodeRadix(nodeCap);
        let guardBits = cap_cnode_cap_get_capCNodeGuardSize(nodeCap);
        let levelBits = radixBits + guardBits;
        let capGuard = cap_cnode_cap_get_capCNodeGuard(nodeCap);
        let guard = (capptr >> ((n_bits - guardBits) & ((1u64<<6) - 1u64))) & ((1u64<<guardBits)-1u64);

        if guardBits > n_bits || guard != capGuard {
            unsafe{
                current_lookup_fault = lookup_fault_guard_mismatch_new(capGuard, n_bits, guardBits);
            }
            ret.status = 2u64;
            return ret;
        }

        if levelBits > n_bits {
            unsafe{
                current_lookup_fault = lookup_fault_depth_mismatch_new(levelBits, n_bits);
            }
            ret.status = 2u64;
            return ret;
        }

        let offset = (capptr >> (n_bits - levelBits)) & ((1<<radixBits)-1);
        let slot = ((cap_cnode_cap_get_capCNodePtr(nodeCap)) + offset) as *mut cte;

        if n_bits == levelBits {
            ret.status = 0u64;
            ret.slot = slot;
            ret.bitsRemaining = 0;
            return ret;
        }

        n_bits = n_bits - levelBits;
        unsafe{
            nodeCap = (*slot).cap;
        }

        if cap_get_capType(nodeCap)!=10u64 {
            ret.status = 0u64;
            ret.slot = slot;
            ret.bitsRemaining = n_bits;
            return ret;
        }
    }
    ret
}

//cap_related
#[no_mangle]
pub extern "C" fn cap_cnode_cap_get_capCNodeRadix(cap:cap_t) -> u64{
    let ret:u64 = (cap.words[0] & 0x1f800000000000u64) >> 47;
    ret
}

#[no_mangle]
pub extern "C" fn cap_cnode_cap_get_capCNodeGuardSize(cap:cap_t) -> u64{
    let ret:u64 = (cap.words[0] & 0x7e0000000000000u64) >> 53;
    ret
}

#[no_mangle]
pub extern "C" fn cap_cnode_cap_get_capCNodeGuard(cap:cap_t) -> u64{
    let ret:u64 = (cap.words[1] & 0xffffffffffffffffu64) >> 0;
    ret
}

#[no_mangle]
pub extern "C" fn cap_get_capType(cap:cap_t) -> u64{
    let ret:u64 = (cap.words[0] >> 59u64) & 0x1fu64;
    ret
}

//error solving
#[no_mangle]
pub extern "C" fn lookup_fault_guard_mismatch_new(guardFound:u64, bitsLeft:u64, bitsFound:u64) -> lookup_fault_t{
    let mut lookup_fault_tmp: lookup_fault_t = lookup_fault_t{
        words:[0,0],
    };
    lookup_fault_tmp.words[0] = 0 | (bitsLeft & 0x7fu64) << 9 | (bitsFound & 0x7fu64) << 2 | (3u64 & 0x3u64) << 0;
    lookup_fault_tmp.words[1] = 0;
    lookup_fault_tmp
}

#[no_mangle]
pub extern "C" fn lookup_fault_depth_mismatch_new(bitsFound:u64, bitsLeft:u64) -> lookup_fault_t{
    let mut lookup_fault_tmp: lookup_fault_t = lookup_fault_t{
        words:[0,0],
    };
    lookup_fault_tmp.words[0] = 0 | (bitsFound & 0x7fu64) << 9 | (bitsLeft & 0x7fu64) << 2 | (2u64 & 0x3u64) << 0;
    lookup_fault_tmp.words[1] = 0;
    lookup_fault_tmp
}

#[no_mangle]
pub extern "C" fn cap_cnode_cap_get_capCNodePtr(cap:cap_t) -> u64{
    let mut ret:u64 = (cap.words[0] & 0x3fffffffff) << 1;
    if (ret & (1u64 << 38))!=0 {
        ret = ret | 0xffffff8000000000;
    }
    ret
}

#[no_mangle]
pub extern "C" fn lookupCap(thread:*mut tcb_t, cPtr:cptr_t) -> lookupCap_ret_t {
    let mut lu_ret: lookupSlot_raw_ret_t = lookupSlot_raw_ret_t{
        status: 0u64,
        slot: 0u64 as *mut cte_t,
    };
    lu_ret = lookupSlot(thread, cPtr);

    let mut ret: lookupCap_ret_t = lookupCap_ret_t{status: 0u64, cap: cap_t{words:[0u64,0u64]}};

    if lu_ret.status != 0u64 {
        ret.status = lu_ret.status;
        ret.cap = cap_null_cap_new();
        return ret;
    }

    ret.status = 0u64;
    unsafe{
        ret.cap = (*lu_ret.slot).cap;
    }
    ret
}

#[no_mangle]
pub extern "C" fn lookupSlot(thread: *mut tcb_t, capptr:cptr_t) -> lookupSlot_raw_ret_t {
    let mut threadRoot:cap_t;
    unsafe{
        threadRoot = (*((((thread as word_t) & !((1u64<<(10)-1)))  + tcb_cnode_index::tcbCTable as u64) as *mut cte_t)).cap;
    }
    let res_ret: resolveAddressBits_ret_t = resolveAddressBits(threadRoot, capptr, 1u64<<6);
    let ret: lookupSlot_raw_ret_t = lookupSlot_raw_ret_t{
        status:res_ret.status,
        slot:res_ret.slot,
    };
    ret
}

//
// #define TCB_PTR_CTE_PTR(p,i) \
//     (((cte_t *)((word_t)(p)&~MASK(seL4_TCBBits)))+(i))

#[no_mangle]
pub extern "C" fn cap_null_cap_new() -> cap_t{
    let cap:cap_t = cap_t{
        words:[0 | (0u64 & 0x1fu64)<<59, 0],
    };
    cap
}

#[no_mangle]
pub extern "C" fn lookupCapAndSlot(thread:*mut tcb_t, cPtr:cptr_t) -> lookupCapAndSlot_ret_t{
    let mut lu_ret: lookupSlot_raw_ret_t = lookupSlot_raw_ret_t{
        status: 0u64,
        slot: 0u64 as *mut cte_t,
    };
    lu_ret = lookupSlot(thread, cPtr);

    let mut ret: lookupCapAndSlot_ret_t = lookupCapAndSlot_ret_t{status: 0u64, cap: cap_t{words:[0u64,0u64]}, slot: 0u64 as *mut cte_t};

    if lu_ret.status != 0u64 {
        ret.status = lu_ret.status;
        ret.slot = 0u64 as *mut cte_t;
        ret.cap = cap_null_cap_new();
        return ret;
    }
    ret.status = 0u64;
    ret.slot = lu_ret.slot;
    unsafe{
        ret.cap = (*lu_ret.slot).cap;
    }
    ret

}

#[no_mangle]
pub extern "C" fn lookupSlotForCNodeOp(isSource:u64, root:cap_t, capptr:cptr_t, depth:word_t) -> lookupSlot_ret_t {
    let mut res_ret: resolveAddressBits_ret_t = resolveAddressBits_ret_t{
        status: 0u64, slot: 0u64 as *mut cte_t, bitsRemaining: 0
    };

    let mut ret: lookupSlot_ret_t = lookupSlot_ret_t{
        status: 0u64,
        slot: 0u64 as *mut cte_t
    };
    //ret.slot = 0u64 as *mut cte_t;

    if cap_get_capType(root) != 10u64 {
        unsafe{
            current_syscall_error.r#type = 6u64;
            current_syscall_error.failedLookupWasSource = isSource;
            current_lookup_fault = lookup_fault_invalid_root_new();
        }
        ret.status = 3;
        return ret;
    }

    if depth < 1 || depth > (1u64<<6) {
        unsafe{
            current_syscall_error.r#type = 4u64;
            current_syscall_error.rangeErrorMin = 1u64;
            current_syscall_error.rangeErrorMax = 1u64<<6;
        }
        ret.status = 3u64;
        return ret;
    }

    if res_ret.bitsRemaining != 0 {
        unsafe{
            current_syscall_error.r#type = 6u64;
            current_syscall_error.failedLookupWasSource = isSource;
            current_lookup_fault = lookup_fault_depth_mismatch_new(0, res_ret.bitsRemaining);
        }
        ret.status = 3u64;
        return ret;
    }

    ret.slot = res_ret.slot;
    ret.status = 0u64;
    ret

}

#[no_mangle]
pub extern "C" fn lookupSourceSlot(root:cap_t, capptr:cptr_t, depth:word_t) -> lookupSlot_ret_t {
    lookupSlotForCNodeOp(true as u64, root, capptr, depth)
}

#[no_mangle]
pub extern "C" fn lookupTargetSlot(root:cap_t, capptr:cptr_t, depth:word_t) -> lookupSlot_ret_t {
    lookupSlotForCNodeOp(false as u64, root, capptr, depth)
}

#[no_mangle]
pub extern "C" fn lookupPivotSlot(root:cap_t, capptr:cptr_t, depth:word_t) -> lookupSlot_ret_t {
    lookupSlotForCNodeOp(true as u64, root, capptr, depth)
}

#[no_mangle]
pub extern "C" fn lookup_fault_invalid_root_new() -> lookup_fault_t{
    let lookup_fault_tmp:lookup_fault_t = lookup_fault_t{
        words: [0 | (0u64 & 0x3u64) << 0, 0],
    };
    lookup_fault_tmp
}
fn main(){

}