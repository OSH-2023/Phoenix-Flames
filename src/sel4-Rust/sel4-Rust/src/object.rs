//to be done
//zrz:2023/7/5 10:00    --update notification-related types and functions, change some (*const point) into (*mut point), which is remain to be discussed later

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]


use crate::machine::registerset::*;
use crate::types::*;

/*
Including contents from:
1. object/structures.h
2. arch/object/structures.h
3. arch/object/structures_gen.h
 */

//1. from object/structures.h

/* Capability table entry (CTE) */
#[derive(Clone,Copy)]
pub struct cte {
    pub cap: cap_t,
    pub cteMDBNode: mdb_node_t,
}
pub type cte_t = cte;

/* TCB: size >= 18 words + sizeof(arch_tcb_t) + 1 word on MCS (aligned to nearest power of 2) */
pub struct tcb {
    /* arch specific tcb state (including context)*/
    tcbArch: arch_tcb_t,

    /* Thread state, 3 words */
    tcbState: thread_state_t,

    /* Notification that this TCB is bound to. If this is set, when this TCB waits on
     * any sync endpoint, it may receive a signal from a Notification object.
     * 1 word*/
    tcbBoundNotification: *mut notification_t,

    /* Current fault, 2 words */
    tcbFault: seL4_Fault_t,

    /* Current lookup failure, 2 words */
    tcbLookupFailure: lookup_fault_t,

    /* Domain, 1 byte (padded to 1 word) */
    tcbDomain: dom_t,

    /*  maximum controlled priority, 1 byte (padded to 1 word) */
    tcbMCP: prio_t,

    /* Priority, 1 byte (padded to 1 word) */
    tcbPriority: prio_t,

    /* Timeslice remaining, 1 word */
    tcbTimeSlice: word_t,

    /* Capability pointer to thread fault handler, 1 word */
    tcbFaultHandler: cptr_t,

    /* userland virtual address of thread IPC buffer, 1 word */
    tcbIPCBuffer: word_t,

    /* Previous and next pointers for scheduler queues , 2 words */
    tcbSchedNext: *const tcb,
    tcbSchedPrev: *const tcb,
    /* Preivous and next pointers for endpoint and notification queues, 2 words */
    tcbEPNext: *mut tcb,
    tcbEPPrev: *const tcb,
}
pub type tcb_t = tcb;

// 2. from arch/object/structures.h
#[derive(Clone,Copy)]
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

// 3. from arch/object/structures_gen.h
#[derive(Clone,Copy)]
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
pub struct thread_state {
    pub words: [u64; 3],
}
pub type thread_state_t = thread_state;

#[derive(Clone,Copy)]
pub struct cap {
    pub words: [u64; 2],
}
pub type cap_t = cap;

#[derive(Clone,Copy)]
pub struct lookup_fault {
    pub words:[u64;2]
}
pub type lookup_fault_t=lookup_fault ;

#[derive(Clone,Copy)]
pub struct seL4_Fault {
    pub words:[u64;2]
}
pub type seL4_Fault_t=seL4_Fault;

// try this
#[no_mangle]
pub extern "C" fn notification_ptr_set_state(notification_ptr: *mut notification_t, v64:u64) {
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
    if (1 & (ret & (1u64 << (38)))) != 0 {
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
    if (1 & (ret & (1u64 << (38)))) != 0 {
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
    if (1 & (ret & (1u64 << (38)))) != 0 {
        ret |= 0xffffff8000000000;
    }
    ret
}

pub fn notification_ptr_get_ntfnMsgIdentifier(notification_ptr: *mut notification_t) -> u64{
    let mut ret:u64;
    unsafe{
        ret = ((*notification_ptr).words[2] & 0xffffffffffffffffu64) >> 0;
    }
    if (1 & (ret & (1u64 << (38)))) != 0 {
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


pub fn cap_null_cap_new()->cap_t{
    let cap:cap_t;

    /* fail if user has passed bits that we will override */  
    //assert(((uint64_t)cap_null_cap & ~0x1full) == ((1 && ((uint64_t)cap_null_cap & (1ull << 47))) ? 0x0 : 0));

    cap.words[0] = 0 | ((cap_null_cap as u64) & 0x1full) << 59;
    cap.words[1] = 0;

    cap;
}
