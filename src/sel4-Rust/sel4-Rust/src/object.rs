//to be done
//zrz:2023/7/5 10:00    --update notification-related types and functions, change some (*const point) into (*mut point), which is remain to be discussed later

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use core::intrinsics::{likely, unlikely};
use crate::machine::registerset::*;
use crate::types::*;
use crate::MASK;

/*
Including contents from:
1. object/structures.h
2. arch/object/structures.h
3. arch/object/structures_gen.h
 */

//1. from object/structures.h

/* Capability table entry (CTE) */
#[derive(Clone,Copy)]
#[repr(C)]
pub struct cte {
    pub cap: cap_t,
    pub cteMDBNode: mdb_node_t,
}
pub type cte_t = cte;

/* TCB: size >= 18 words + sizeof(arch_tcb_t) + 1 word on MCS (aligned to nearest power of 2) */
#[derive(Clone,Copy)]
#[repr(C)]
pub struct tcb {
    /* arch specific tcb state (including context)*/
    pub tcbArch: arch_tcb_t,

    /* Thread state, 3 words */
    pub tcbState: thread_state_t,

    /* Notification that this TCB is bound to. If this is set, when this TCB waits on
     * any sync endpoint, it may receive a signal from a Notification object.
     * 1 word*/
     pub tcbBoundNotification: *mut notification_t,

    /* Current fault, 2 words */
    pub tcbFault: seL4_Fault_t,

    /* Current lookup failure, 2 words */
    pub tcbLookupFailure: lookup_fault_t,

    /* Domain, 1 byte (padded to 1 word) */
    pub tcbDomain: dom_t,

    /*  maximum controlled priority, 1 byte (padded to 1 word) */
    pub tcbMCP: prio_t,

    /* Priority, 1 byte (padded to 1 word) */
    pub tcbPriority: prio_t,

    /* Timeslice remaining, 1 word */
    pub tcbTimeSlice: word_t,

    /* Capability pointer to thread fault handler, 1 word */
    pub tcbFaultHandler: cptr_t,

    /* userland virtual address of thread IPC buffer, 1 word */
    pub tcbIPCBuffer: word_t,

    /* Previous and next pointers for scheduler queues , 2 words */
    pub tcbSchedNext: *const tcb,
    pub tcbSchedPrev: *const tcb,
    /* Preivous and next pointers for endpoint and notification queues, 2 words */
    pub tcbEPNext: *mut tcb,
    pub tcbEPPrev: *const tcb,
}
pub type tcb_t = tcb;

#[macro_export]
macro_rules! CTE_PTR {
    ($r:expr) => {
        (($r) as *mut cte_t)
    };
}

pub fn TCB_PTR_CTE_PTR(p:*mut tcb_t, i:u64) -> *mut cte_t{
    ((((p as word_t) & (!MASK!(10))) as u64) + i) as *mut cte_t
}

// 2. from arch/object/structures.h
#[derive(Clone,Copy)]
#[repr(C)]
pub struct arch_tcb {
    pub tcbContext: user_context_t,
}
pub type arch_tcb_t = arch_tcb;

#[derive(Clone,Copy)]
pub enum notification_state{
    NtfnState_Idle = 0,
    NtfnState_Waiting = 1,
    NtfnState_Active = 2,
}
pub type notification_state_t = word_t;

#[derive(Clone,Copy)]
pub enum _thread_state{
    ThreadState_Inactive = 0,
    ThreadState_Running,
    ThreadState_Restart,
    ThreadState_BlockedOnReceive,
    ThreadState_BlockedOnSend,
    ThreadState_BlockedOnReply,
    ThreadState_BlockedOnNotification,
    ThreadState_IdleThreadState,
}
pub type _thread_state_t = word_t;

pub enum tcb_cnode_index {
    /* CSpace root */
    tcbCTable = 0,

    /* VSpace root */
    tcbVTable = 1,
    /* Reply cap slot */
    tcbReply = 2,

    /* TCB of most recent IPC sender */
    tcbCaller = 3,

    /* IPC buffer cap slot */
    tcbBuffer = 4,
    tcbCNodeEntries
}


// 3. from arch/object/structures_gen.h
#[derive(Clone,Copy)]
#[repr(C)]
pub struct mdb_node {
    pub words: [u64; 2],
}
pub type mdb_node_t = mdb_node;

#[derive(Clone,Copy)]
#[repr(C)]
pub struct notification {
    pub words:[u64;4]
}
pub type notification_t=notification;

#[derive(Clone,Copy)]
#[repr(C)]
pub struct thread_state {
    pub words: [u64; 3],
}
pub type thread_state_t = thread_state;

#[derive(Clone,Copy)]
#[repr(C)]
pub struct cap {
    pub words: [u64; 2],
}
pub type cap_t = cap;

#[derive(Clone,Copy)]
#[repr(C)]
pub struct lookup_fault {
    pub words:[u64;2]
}
pub type lookup_fault_t=lookup_fault ;

#[derive(Clone,Copy)]
#[repr(C)]
pub struct seL4_Fault {
    pub words:[u64;2]
}
pub type seL4_Fault_t=seL4_Fault;


pub fn notification_ptr_set_state(notification_ptr: *mut notification_t, v64:u64) {
    //assert_eq!((((!0x3u64 >> 0) | 0x0) & v64) ,  if false && (v64 & (1u64 << (38))) != 0 { 0x0u64 }else{ 0u64 }, "seL4 failed assertion 'object.rs' at :line 120 in function notification_ptr_set_state\n");
    unsafe{
        (*notification_ptr).words[0] &= !0x3u64;
        (*notification_ptr).words[0] |= (v64 << 0) & 0x3u64;
    }
}

pub fn notification_ptr_get_state(notification_ptr: *mut notification_t) -> u64{
    let mut ret:u64 ;
    unsafe{
        ret = ((*notification_ptr).words[0] & 0x3u64) >> 0;
    }
    ret
}

pub fn notification_ptr_get_ntfnQueue_head(notification_ptr:*mut notification_t) ->u64 {
    let mut ret:u64 = 0;
    unsafe{
        ret = ((*notification_ptr).words[0] & 0xfffffffffe000000u64) >> 25;
    }
    if (true && (ret & (1u64 << (38)))) != 0 {
        ret = ret | 0xffffff8000000000;
    }
    ret
}

pub fn notification_ptr_set_ntfnQueue_head(notification_ptr:*mut notification_t, v64:u64) {
    //     assert((((~0x7fffffffffull << 0) | 0xffffff8000000000) & v64) == ((1 && (v64 & (1ull << (38)))) ? 0xffffff8000000000 : 0));
    unsafe{
        (*notification_ptr).words[1] &= !0x7fffffffffu64;
        (*notification_ptr).words[1] |= (v64 >> 0) & 0x7fffffffff;
    }
}

pub fn notification_ptr_get_ntfnQueue_tail(notification_ptr:*mut notification_t) -> u64 {
    let mut ret:u64 = 0;
    unsafe{
        ret = ((*notification_ptr).words[0] & 0xfffffffffe000000u64) >> 25;
    }
    if (true && (ret & (1u64 << (38)))) != 0 {
        ret |= 0xffffff8000000000;
    }
    ret
}

pub fn notification_ptr_set_ntfnQueue_tail(notification_ptr:*mut notification_t, v64:u64) {
    //     assert((((~0xfffffffffe000000ull >> 25) | 0xffffff8000000000) & v64) == ((1 && (v64 & (1ull << (38)))) ? 0xffffff8000000000 : 0));
    unsafe{
        (*notification_ptr).words[0] &= !0xfffffffffe000000u64;
        (*notification_ptr).words[0] |= (v64 << 25) & 0xfffffffffe000000;
    }
}

pub fn notification_ptr_get_ntfnBoundTCB(notification_ptr:*mut notification_t) -> u64 {
    let mut ret:u64;
    unsafe{
        ret = ((*notification_ptr).words[3] & 0x7fffffffffu64 ) << 0;
    }
    if (true && (ret & (1u64 << (38)))) != 0 {
        ret |= 0xffffff8000000000;
    }
    ret
}

pub fn notification_ptr_get_ntfnMsgIdentifier(notification_ptr: *mut notification_t) -> u64{
    let mut ret:u64;
    unsafe{
        ret = ((*notification_ptr).words[2] & 0xffffffffffffffffu64) >> 0;
    }
    if (true && (ret & (1u64 << (38)))) != 0 {
        ret |= 0x0;
    }
    ret
}

pub fn notification_ptr_set_ntfnMsgIdentifier(notification_ptr:*mut notification_t, v64:u64) {
//     assert((((~0xffffffffffffffffull >> 0) | 0x0) & v64) == ((0 && (v64 & (1ull << (38)))) ? 0x0 : 0));
    unsafe{
        (*notification_ptr).words[2] &= !0xffffffffffffffffu64;
        (*notification_ptr).words[2] |= (v64 << 0) & 0xffffffffffffffff;
    }    
}

pub fn notification_ptr_set_ntfnBoundTCB(notification_ptr:*mut notification_t, v64:u64) {
    //     assert((((~0x7fffffffffull << 0) | 0xffffff8000000000) & v64) == ((1 && (v64 & (1ull << (38)))) ? 0xffffff8000000000 : 0));
    unsafe{
        (*notification_ptr).words[3] &= !0x7fffffffffu64;
        (*notification_ptr).words[3] |= (v64>>0) & 0x7fffffffff;
    }
}

#[derive(Clone, Copy)]
pub enum lookup_fault_tag {
    lookup_fault_invalid_root = 0,
    lookup_fault_missing_capability = 1,
    lookup_fault_depth_mismatch = 2,
    lookup_fault_guard_mismatch = 3
}
pub type lookup_fault_tag_t=lookup_fault_tag;

#[derive(Clone, Copy)]
pub enum cap_tag {
    cap_null_cap = 0,
    cap_untyped_cap = 2,
    cap_endpoint_cap = 4,
    cap_notification_cap = 6,
    cap_reply_cap = 8,
    cap_cnode_cap = 10,
    cap_thread_cap = 12,
    cap_irq_control_cap = 14,
    cap_irq_handler_cap = 16,
    cap_zombie_cap = 18,
    cap_domain_cap = 20,
    cap_frame_cap = 1,
    cap_page_table_cap = 3,
    cap_page_directory_cap = 5,
    cap_pdpt_cap = 7,
    cap_pml4_cap = 9,
    cap_asid_control_cap = 11,
    cap_asid_pool_cap = 13,
    cap_io_port_cap = 19,
    cap_io_port_control_cap = 31
}
pub type  cap_tag_t=cap_tag;

#[inline(always)]
pub fn cap_null_cap_new()->cap_t{
    let mut cap=cap_t{
        words:[0,0]
    };

    /* fail if user has passed bits that we will override */  
    //assert(((uint64_t)cap_null_cap & ~0x1full) == ((1 && ((uint64_t)cap_null_cap & (1ull << 47))) ? 0x0 : 0));

    cap.words[0] = 0 | ((0 as u64) & 0x1fu64) << 59;
    cap.words[1] = 0;

    cap
}

#[inline(always)]
pub fn cap_get_capType(cap:cap_t)->u64 {
    (cap.words[0] >> 59) & 0x1fu64
}

#[inline(always)]
pub fn lookup_fault_invalid_root_new()->lookup_fault_t {
    let mut lookup_fault=lookup_fault_t{
        words:[0,0]
    };

    /* fail if user has passed bits that we will override */  
    //assert(((uint64_t)lookup_fault_invalid_root & ~0x3ull) == ((1 && ((uint64_t)lookup_fault_invalid_root & (1ull << 47))) ? 0x0 : 0));

    lookup_fault.words[0] = 0
        | (0 as u64 & 0x3u64) << 0;
    lookup_fault.words[1] = 0;

    lookup_fault
}

 #[inline(always)]
pub fn lookup_fault_depth_mismatch_new(bitsFound:u64, bitsLeft:u64)->lookup_fault_t {
    let mut lookup_fault=lookup_fault_t{
        words:[0,0]
    };

    /* fail if user has passed bits that we will override */  
    // assert((bitsFound & ~0x7full) == ((1 && (bitsFound & (1ull << 47))) ? 0x0 : 0));  
    // assert((bitsLeft & ~0x7full) == ((1 && (bitsLeft & (1ull << 47))) ? 0x0 : 0));  
    // assert(((uint64_t)lookup_fault_depth_mismatch & ~0x3ull) == ((1 && ((uint64_t)lookup_fault_depth_mismatch & (1ull << 47))) ? 0x0 : 0));

    lookup_fault.words[0] = 0
        | (bitsFound & 0x7fu64) << 9
        | (bitsLeft & 0x7fu64) << 2
        | (lookup_fault_tag::lookup_fault_depth_mismatch as u64& 0x3u64) << 0;
    lookup_fault.words[1] = 0;

    lookup_fault
}

#[inline(always)]
pub fn cap_cnode_cap_get_capCNodeRadix(cap:cap_t)->u64 {
    let mut ret:u64;
    // assert(((cap.words[0] >> 59) & 0x1f) ==
    //        cap_cnode_cap);

    ret = (cap.words[0] & 0x1f800000000000u64) >> 47;
    ret
}

#[inline(always)]
pub fn cap_cnode_cap_get_capCNodeGuardSize(cap:cap_t)->u64 {
    let mut ret:u64;
    // assert(((cap.words[0] >> 59) & 0x1f) ==
    //        cap_cnode_cap);

    ret = (cap.words[0] & 0x7e0000000000000u64) >> 53;
    ret
}

#[inline(always)]
pub fn cap_cnode_cap_get_capCNodeGuard(cap:cap_t)->u64 {
    let mut ret:u64;
    // assert(((cap.words[0] >> 59) & 0x1f) ==
    //        cap_cnode_cap);

    ret = (cap.words[1] & 0xffffffffffffffffu64) >> 0;
    ret
}


#[inline(always)]
pub fn lookup_fault_guard_mismatch_new( guardFound:u64,  bitsLeft:u64,  bitsFound:u64)->lookup_fault_t {
    let mut lookup_fault=lookup_fault_t{
        words:[0,0]
    };

    /* fail if user has passed bits that we will override */ 
    // assert((bitsLeft & ~0x7full) == ((1 && (bitsLeft & (1ull << 47))) ? 0x0 : 0));  
    // assert((bitsFound & ~0x7full) == ((1 && (bitsFound & (1ull << 47))) ? 0x0 : 0));  
    // assert(((uint64_t)lookup_fault_guard_mismatch & ~0x3ull) == ((1 && ((uint64_t)lookup_fault_guard_mismatch & (1ull << 47))) ? 0x0 : 0));

    lookup_fault.words[0] = 0
        | (bitsLeft & 0x7fu64) << 9
        | (bitsFound & 0x7fu64) << 2
        | (lookup_fault_tag::lookup_fault_guard_mismatch as u64& 0x3u64) << 0;
    lookup_fault.words[1] = 0
        | guardFound << 0;

    lookup_fault
}

#[inline(always)]
pub fn cap_cnode_cap_get_capCNodePtr( cap:cap_t) ->u64{
    let mut ret:u64;
    // assert(((cap.words[0] >> 59) & 0x1f) ==
    //        cap_cnode_cap);

    ret = (cap.words[0] & 0x7fffffffffffu64) << 1;
    /* Possibly sign extend */
    if likely((ret & (1u64 << (47)))!=0) {
        ret |= 0xffff000000000000;
    }
    return ret;
}

#[inline(always)]
pub fn thread_state_ptr_get_tsType(thread_state_ptr:*mut thread_state_t) -> u64 {
    unsafe{
        let ret:u64 = ((*thread_state_ptr).words[0] & 0xfu64) >> 0;
    }
}

#[inline(always)]
pub fn cap_notification_cap_get_capNtfnPtr(cap:cap_t) -> u64 {
    let mut ret:u64 = (cap.words[0] & 0x7fffffffffu64);
    if true && (ret & (1u64 << (38))) {
        ret |= 0xffffff8000000000;
    }
    ret
}

#[inline(always)]
pub fn thread_state_ptr_set_blockingObject(thread_state_ptr:*mut thread_state_t, v64:u64) {
    unsafe{
        (*thread_state_ptr).words[0] &= !0x7ffffffff0u64;
        (*thread_state_ptr).words[0] |= (v64 >> 0) & 0x7ffffffff0;
    }
}

//from constants.h
pub enum priorityConstants {
    seL4_InvalidPrio = -1,
    seL4_MinPrio = 0,
    seL4_MaxPrio = 255,
}