//zrz 2023-6-11 12:00
//zrz 2023-7-5 10:00                --some related functions are from thread and tcb
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::kernel::fastpath::_thread_state::*;
use crate::kernel::fastpath::endpoint_state::*;
use crate::kernel::tcb::*;
use crate::kernel::thread::*;
use crate::object::_thread_state::ThreadState_BlockedOnNotification;
use crate::object::_thread_state::ThreadState_BlockedOnReceive;
use crate::object::_thread_state::ThreadState_Inactive;
use crate::object::_thread_state::ThreadState_Restart;
use crate::object::_thread_state::ThreadState_Running;
use crate::object::notification_state::NtfnState_Active;
use crate::object::notification_state::NtfnState_Idle;
use crate::object::notification_state::NtfnState_Waiting;
use crate::object::tcb_cnode_index::*;
use crate::object::*;
use crate::object::{
    notification_ptr_get_ntfnQueue_head, notification_ptr_get_ntfnQueue_tail,
    notification_ptr_get_state, notification_ptr_set_ntfnQueue_head,
    notification_ptr_set_ntfnQueue_tail, notification_ptr_set_state,
};
use crate::types::word_t;

extern "C" {
    pub fn cteDeleteOne(slot: *mut cte);
}

pub fn ntfn_ptr_get_queue(ntfnPtr: *mut notification_t) -> tcb_queue_t {
    let ntfn_queue: tcb_queue_t = tcb_queue_t {
        head: notification_ptr_get_ntfnQueue_head(ntfnPtr) as (*mut tcb_t),
        end: notification_ptr_get_ntfnQueue_tail(ntfnPtr) as (*mut tcb_t),
    };
    ntfn_queue
}

pub fn ntfn_ptr_set_queue(ntfnPtr: *mut notification_t, ntfn_queue: tcb_queue_t) {
    notification_ptr_set_ntfnQueue_head(ntfnPtr, ntfn_queue.head as word_t);
    notification_ptr_set_ntfnQueue_tail(ntfnPtr, ntfn_queue.end as word_t);
}

pub fn sendSignal(ntfnPtr: *mut notification_t, badge: word_t) {
    let tmp: u64 = notification_ptr_get_state(ntfnPtr);
    if tmp == NtfnState_Idle as u64 {
        let mut tcb: *mut tcb_t = notification_ptr_get_ntfnBoundTCB(ntfnPtr) as (*mut tcb_t);
        if tcb != 0 as *mut tcb {
            if unsafe {
                thread_state_ptr_get_tsType(&mut (*tcb).tcbState)
                    == ThreadState_BlockedOnReceive as u64
            } {
                cancelIPC(tcb); //from endpoint.c
                unsafe {
                    setThreadState(tcb, ThreadState_Running as u64); //from thread.c
                }
                unsafe {
                    setRegister(tcb, /*badgeRegister*/ 9, badge); //from thread.c
                    possibleSwitchTo(tcb); //from thread.c
                }
            } else {
                ntfn_set_active(ntfnPtr, badge);
            }
        } else {
            ntfn_set_active(ntfnPtr, badge);
        }
        //break;
    } else if tmp == NtfnState_Waiting as u64 {
        let mut nftn_queue: tcb_queue_t = ntfn_ptr_get_queue(ntfnPtr);
        let mut dest: *mut tcb_t = nftn_queue.head;
        //assert(dest) //this is for failure detection
        nftn_queue = tcbEPDequeue(dest, &mut nftn_queue as *mut tcb_queue_t); //from tcb.c
        ntfn_ptr_set_queue(ntfnPtr, nftn_queue);
        if nftn_queue.head == 0 as *mut tcb_t {
            notification_ptr_set_state(ntfnPtr, NtfnState_Idle as u64);
        }
        unsafe {
            setThreadState(dest, ThreadState_Running as u64);
        }
        unsafe {
            setRegister(dest, badgeRegister, badge);
            possibleSwitchTo(dest);
        }
    } else if tmp == NtfnState_Active as u64 {
        let badge2: word_t = notification_ptr_get_ntfnMsgIdentifier(ntfnPtr) | badge;
        notification_ptr_set_ntfnMsgIdentifier(ntfnPtr, badge2);
    }
}

pub fn receiveSignal(thread: *mut tcb_t, cap: cap_t, isBlocking: word_t) {
    let mut ntfnPtr: *mut notification_t =
        cap_notification_cap_get_capNtfnPtr(cap) as (*mut notification_t);
    let tmp: u64 = notification_ptr_get_state(ntfnPtr);
    if tmp == NtfnState_Waiting as u64 {
        let mut ntfn_queue: tcb_queue_t;
        if isBlocking != 0 {
            unsafe {
                thread_state_ptr_set_tsType(
                    &mut (*thread).tcbState,
                    ThreadState_BlockedOnNotification as u64,
                );
                thread_state_ptr_set_blockingObject(&mut (*thread).tcbState, ntfnPtr as word_t);
            }
            unsafe {
                scheduleTCB(thread); //from thread.c
            }
            ntfn_queue = ntfn_ptr_get_queue(ntfnPtr);
            ntfn_queue = tcbEPAppend(thread, ntfn_queue);
            notification_ptr_set_state(ntfnPtr, NtfnState_Waiting as u64);
            ntfn_ptr_set_queue(ntfnPtr, ntfn_queue);
        } else {
            unsafe {
                doNBRecvFailedTransfer(&mut *thread);
            }
        }
    } else if tmp == NtfnState_Active as u64 {
        unsafe {
            setRegister(
                thread,
                badgeRegister,
                notification_ptr_get_ntfnMsgIdentifier(ntfnPtr),
            );
        }
        notification_ptr_set_state(ntfnPtr, NtfnState_Idle as u64);
    }
}

pub fn cancleAllSignals(ntfnPtr: *mut notification_t) {
    if notification_ptr_get_state(ntfnPtr) == NtfnState_Waiting as u64 {
        let mut thread: *mut tcb_t = notification_ptr_get_ntfnQueue_head(ntfnPtr) as (*mut tcb_t);
        notification_ptr_set_state(ntfnPtr, NtfnState_Idle as u64);
        notification_ptr_set_ntfnQueue_head(ntfnPtr, 0);
        notification_ptr_set_ntfnQueue_tail(ntfnPtr, 0);
        while thread as u64 != 0 {
            unsafe {
                setThreadState(thread, ThreadState_Restart as u64); //from thread.c
            }
            tcbSchedEnqueue(thread); // from tcb.c
            rescheduleRequired();
            unsafe {
                thread = (*thread).tcbEPNext; //from thread.c
            }
        }
    }
}

pub fn cancelSignal(threadPtr: *mut tcb_t, ntfnPtr: *mut notification_t) {
    let mut ntfn_queue: tcb_queue_t;
    //     assert(notification_ptr_get_state(ntfnPtr) == NtfnState_Waiting);

    ntfn_queue = ntfn_ptr_get_queue(ntfnPtr);
    ntfn_queue = tcbEPDequeue(threadPtr, &mut ntfn_queue as *mut tcb_queue);
    ntfn_ptr_set_queue(ntfnPtr, ntfn_queue);

    if ntfn_queue.head == core::ptr::null_mut() {
        notification_ptr_set_state(ntfnPtr, NtfnState_Idle as u64);
    }

    unsafe {
        setThreadState(threadPtr, ThreadState_Inactive as u64);
    }
}

pub fn completeSignal(ntfnPtr: *mut notification_t, tcb: *mut tcb_t) {
    let mut badge: word_t;

    if tcb as u64 != 0 && notification_ptr_get_state(ntfnPtr) == NtfnState_Active as u64 {
        badge = notification_ptr_get_ntfnMsgIdentifier(ntfnPtr);
        unsafe {
            setRegister(tcb, badgeRegister, badge);
        }
        notification_ptr_set_state(ntfnPtr, NtfnState_Idle as u64);
    } else {
        panic!("tried to complete signal with inactive notification object"); // here we use the macro panic, which we might need to define later
    }
}

pub fn doUnbindNotification(ntfnPtr: *mut notification_t, tcbptr: *mut tcb_t) {
    notification_ptr_set_ntfnBoundTCB(ntfnPtr, 0 as word_t);
    unsafe {
        (*tcbptr).tcbBoundNotification = 0u64 as (*mut notification_t);
    }
}

pub fn unbindMaybeNotification(ntfnPtr: *mut notification_t) {
    let boundTCB: *mut tcb_t = notification_ptr_get_ntfnBoundTCB(ntfnPtr) as *mut tcb_t;
    if boundTCB as u64 != 0 {
        doUnbindNotification(ntfnPtr, boundTCB);
    }
}

pub fn unbindNotification(tcb: *mut tcb_t) {
    let ntfnPtr: *mut notification_t;
    unsafe {
        ntfnPtr = (*tcb).tcbBoundNotification;
    }
    if ntfnPtr as u64 != 0 {
        doUnbindNotification(ntfnPtr, tcb);
    }
}

pub fn bindNotification(tcb: *mut tcb_t, ntfnPtr: *mut notification_t) {
    notification_ptr_set_ntfnBoundTCB(ntfnPtr, tcb as word_t);
    unsafe {
        (*tcb).tcbBoundNotification = ntfnPtr;
    }
}

pub fn ntfn_set_active(ntfnPtr: *mut notification_t, badge: word_t) {
    notification_ptr_set_state(ntfnPtr, NtfnState_Active as u64);
    notification_ptr_set_ntfnMsgIdentifier(ntfnPtr, badge);
}

//belows are from endpoint.c, I add it here since nobody is in charge of endpoint.c
pub fn cancelIPC(tptr: *mut tcb_t) {
    let mut state: thread_state_t;
    unsafe {
        state = (*tptr).tcbState;
    }
    let tmp: u64 = thread_state_ptr_get_tsType(&mut state as *mut thread_state);
    if tmp == ThreadState_BlockedOnSend as u64 {
    } else if tmp == ThreadState_BlockedOnReceive as u64 {
        let mut epptr: *mut endpoint_t;
        let mut queue: tcb_queue_t;
        epptr =
            thread_state_ptr_get_blockingObject(&mut state as *mut thread_state) as *mut endpoint_t;
        queue = ep_ptr_get_queue(epptr);
        queue = tcbEPDequeue(tptr, &mut queue as *mut tcb_queue);
        ep_ptr_set_queue(epptr, queue);
        if queue.head == 0 as *mut tcb_t {
            endpoint_ptr_set_state(epptr, EPState_Idle as u64);
        }
        setThreadState(tptr, ThreadState_Inactive as u64);
    } else if tmp == ThreadState_BlockedOnNotification as u64 {
        cancelSignal(
            tptr,
            thread_state_ptr_get_blockingObject(&mut state as *mut thread_state)
                as *mut notification_t,
        );
    } else if tmp == ThreadState_BlockedOnReply as u64 {
        let mut slot: *mut cte_t;
        let mut callerCap: *mut cte_t;
        unsafe {
            (*tptr).tcbFault = seL4_Fault_NullFault_new();
        }
        slot = TCB_PTR_CTE_PTR(tptr, tcbReply as u64);
        unsafe {
            callerCap = mdb_node_get_mdbNext((*slot).cteMDBNode) as *mut cte_t;
        }
        if callerCap != core::ptr::null_mut() {
            unsafe {
                cteDeleteOne(callerCap);
            }
        }
    }
}

#[inline(always)]
pub fn ep_ptr_get_queue(epptr: *mut endpoint_t) -> tcb_queue_t {
    tcb_queue_t{
        head:endpoint_ptr_get_epQueue_head(epptr) as *mut tcb_t,
        end:endpoint_ptr_get_epQueue_tail(epptr) as *mut tcb_t
    }
}

#[inline(always)]
pub fn ep_ptr_set_queue(epptr: *mut endpoint_t, queue: tcb_queue_t) {
    endpoint_ptr_set_epQueue_head(epptr, queue.head as word_t);
    endpoint_ptr_set_epQueue_tail(epptr, queue.end as word_t);
}

#[inline(always)]
pub fn endpoint_ptr_get_epQueue_head(endpoint_ptr: *mut endpoint_t) -> u64 {
    let mut ret: u64;
    unsafe {
        ret = ((*endpoint_ptr).words[1] & 0xffffffffffffffffu64) >> 0;
    }
    ret
}

#[inline(always)]
pub fn endpoint_ptr_get_epQueue_tail(endpoint_ptr: *mut endpoint_t) -> u64 {
    let mut ret: u64;
    unsafe {
        ret = ((*endpoint_ptr).words[0] & 0x7ffffffffcu64) >> 0;
    }
    if true && (ret & (1u64 << (38)) != 0) {
        ret |= 0xffffff8000000000;
    }
    ret
}

#[inline(always)]
pub fn endpoint_ptr_set_epQueue_head(endpoint_ptr: *mut endpoint_t, v64: u64) {
    unsafe {
        (*endpoint_ptr).words[1] &= !0xffffffffffffffffu64;
        (*endpoint_ptr).words[1] |= (v64 << 0) & 0xffffffffffffffff;
    }
}

#[inline(always)]
pub fn endpoint_ptr_set_epQueue_tail(endpoint_ptr: *mut endpoint_t, v64: u64) {
    unsafe {
        (*endpoint_ptr).words[0] &= !0x7ffffffffcu64;
        (*endpoint_ptr).words[0] |= (v64 >> 0) & 0x7ffffffffc;
    }
}

#[inline(always)]
pub fn endpoint_ptr_set_state(endpoint_ptr: *mut endpoint_t, v64: u64) {
    unsafe {
        (*endpoint_ptr).words[0] &= !0x3u64;
        (*endpoint_ptr).words[0] |= (v64 << 0) & 0x3;
    }
}

#[inline(always)]
pub fn mdb_node_get_mdbNext(mdb_nod: mdb_node_t) -> u64 {
    let mut ret: u64 = (mdb_nod.words[0] & 0x7ffffffffcu64) << 0;
    if true && (ret & (1u64 << (38)) != 0) {
        ret |= 0xffffff8000000000;
    }
    ret
}
