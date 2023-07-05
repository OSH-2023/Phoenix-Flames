//zrz 2023-6-11 12:00
//zrz 2023-7-5 10:00                --some related functions are from thread and tcb
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::object::*;
use crate::types::word_t;
use crate::object::_thread_state::ThreadState_BlockedOnReceive;
use crate::object::_thread_state::ThreadState_Running;
use crate::object::notification_state::NtfnState_Idle;
use crate::object::_thread_state::ThreadState_BlockedOnNotification;
use crate::object::notification_state::NtfnState_Waiting;
use crate::object::_thread_state::ThreadState_Restart;
use crate::object::_thread_state::ThreadState_Inactive;
use crate::object::notification_state::NtfnState_Active;
use crate::object::{notification_ptr_set_state,notification_ptr_get_state,notification_ptr_get_ntfnQueue_head,notification_ptr_set_ntfnQueue_head,notification_ptr_get_ntfnQueue_tail,notification_ptr_set_ntfnQueue_tail};


pub fn ntfn_ptr_get_queue(ntfnPtr: *mut notification_t) -> tcb_queue_t {
    let ntfn_queue:tcb_queue_t = tcb_queue_t{
        head: notification_ptr_get_ntfnQueue_head(ntfnPtr) as (*mut tcb_t),
        end: notification_ptr_get_ntfnQueue_tail(ntfnPtr) as (*mut tcb_t),
    };
    ntfn_queue
}

pub fn ntfn_ptr_set_queue(ntfnPtr: *mut notification_t, ntfn_queue: tcb_queue_t) {
    notification_ptr_set_ntfnQueue_head(ntfnPtr, ntfn_queue.head as word_t);
    notification_ptr_set_ntfnQueue_tail(ntfnPtr, ntfn_queue.tail as word_t);
}

pub fn sendSignal(ntfnPtr: *mut notification_t, badge: word_t) {
    let tmp:u64 = notification_ptr_get_state(ntfnPtr);
    if tmp == NtfnState_Idle as u64 {
        let mut tcb: *mut tcb_t = notification_ptr_get_ntfnBoundTCB(ntfnPtr) as (*mut tcb_t);
        if tcb!=0 {
            if unsafe{thread_state_ptr_get_tsType((*tcb).tcbState) == ThreadState_BlockedOnReceive} {
                cancleIPC(tcb);//from endpoint.c
                setThreadState(tcb, ThreadState_Running);//from thread.c
                setRegister(tcb, /*badgeRegister*/9, badge);//from thread.c
                possibleSwitchTo(tcb);//from thread.c
            } else {
                ntfn_set_active(ntfnPtr, badge);
            }
        } else {
            ntfn_set_active(ntfnPtr, badge);
        }
        //break;
    }else
    if tmp == NtfnState_Waiting as u64 {
        let mut nftn_queue: tcb_queue_t = ntfn_ptr_get_queue(ntfnPtr);
        let mut dest: *mut tcb_t = nftn_queue.head;
        //assert(dest) //this is for failure detection
        nftn_queue = tcbEPDequeue(nftn_queue, dest);//from tcb.c
        ntfn_ptr_set_queue(ntfnPtr, nftn_queue);
        if !nftn_queue.head {
            notification_ptr_set_state(ntfnPtr, NtfnState_Idle);
        }
        setThreadState(dest, ThreadState_Running);
        setRegister(dest, badgeRegister, badge);
        possibleSwitchTo(dest);
    }else
    if tmp == NtfnState_Active as u64 {
        let badge2:word_t = notification_ptr_get_ntfnMsgIdentifier(ntfnPtr) | badge;
        notification_ptr_set_ntfnMsgIdentifier(ntfnPtr, badge2);
    }
}

pub fn receiveSignal(thread: *mut tcb_t, cap: cap_t, isBlocking: word_t) {
    let mut ntfnPtr: *mut notification_t = cap_notification_cap_get_capNtfnPtr(cap) as (*mut notification_t);
    let tmp:u64 = notification_ptr_get_state(ntfnPtr);
    if tmp == NtfnState_Waiting as u64{
        let mut ntfn_queue:tcb_queue_t;
        if isBlocking!=0 {
            unsafe{
                thread_state_ptr_set_tsType((*thread).tcbState, ThreadState_BlockedOnNotification as u64);
                thread_state_ptr_set_blockingObject((*thread).tcbState, ntfnPtr as word_t);
            }
            scheduleTCB(thread); //from thread.c
            ntfn_queue = ntfn_ptr_get_queue(ntfnPtr);
            ntfn_queue = tcbEPAppend(thread, ntfn_queue);
            notification_ptr_set_state(ntfnPtr, NtfnState_Waiting as u64);
            ntfn_ptr_set_queue(ntfnPtr, ntfn_queue);
        } else {
            doNBRecvFailedTransfer(thread);
        }
    }else
    if tmp == NtfnState_Active as u64{
        setRegister(thread, badgeRegister, notification_ptr_get_ntfnMsgIdentifier(ntfnPtr));
        notification_ptr_set_state(ntfnPtr, NtfnState_Idle as u64);

    }
}


pub fn cancleAllSignals(ntfnPtr: *mut notification_t) {
    if notification_ptr_get_state(ntfnPtr) == NtfnState_Waiting as u64 {
        let mut thread:* mut tcb_t = notification_ptr_get_ntfnQueue_head(ntfnPtr) as (*mut tcb_t);
        notification_ptr_set_state(ntfnPtr, NtfnState_Idle as u64);
        notification_ptr_set_ntfnQueue_head(ntfnPtr, 0);
        notification_ptr_set_ntfnQueue_tail(ntfnPtr, 0);
        while thread as u64 != 0 {
            setThreadState(thread, ThreadState_Restart);//from thread.c
            tcbSchedEnqueue(thread);// from tcb.c
            rescheduleRequired();
            unsafe {
                thread = (*thread).tcbEPNext;//from thread.c
            }
        }

    }
}

pub fn cancelSignal(threadPtr:*mut tcb_t, ntfnPtr:*mut notification_t) {
    let mut ntfn_queue:tcb_queue_t;
    //     assert(notification_ptr_get_state(ntfnPtr) == NtfnState_Waiting);

    ntfn_queue = ntfn_ptr_get_queue(ntfnPtr);
    ntfn_queue = tcbEPDequeue(threadPtr, ntfn_queue);
    ntfn_ptr_set_queue(ntfnPtr, ntfn_queue);

    if ntfn_queue.head == 0 {
        notification_ptr_set_state(ntfnPtr, NtfnState_Idle as u64);
    }

    setThreadState(threadPtr, ThreadState_Inactive);

}

pub fn completeSignal(ntfnPtr:*mut notification_t, tcb:*mut tcb_t) {
    let mut badge:word_t;
    
    if tcb as u64!=0 && notification_ptr_get_state(ntfnPtr) == NtfnState_Active as u64{
        badge = notification_ptr_get_ntfnMsgIdentifier(ntfnPtr);
        setRegister(tcb, badgeRegister, badge);
        notification_ptr_set_state(ntfnPtr, NtfnState_Idle as u64);       
    } else {
        panic!("tried to complete signal with inactive notification object");// here we use the macro panic, which we might need to define later
    }
}

pub fn doUnbindNotification(ntfnPtr:*mut notification_t, tcbptr: *mut tcb_t) {
    notification_ptr_set_ntfnBoundTCB(ntfnPtr, 0 as word_t);
    unsafe{
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
    let ntfnPtr:*mut notification_t;
    unsafe{
        ntfnPtr = (*tcb).tcbBoundNotification;
    }
    if ntfnPtr as u64!=0 {
        doUnbindNotification(ntfnPtr, tcb);
    }
}

pub fn bindNotification(tcb:*mut tcb_t, ntfnPtr:*mut notification_t){
    notification_ptr_set_ntfnBoundTCB(ntfnPtr, tcb as word_t);
    unsafe{
        (*tcb).tcbBoundNotification = ntfnPtr;
    }
}

pub fn ntfn_set_active(ntfnPtr: *mut notification_t, badge: word_t){
    notification_ptr_set_state(ntfnPtr, NtfnState_Active as u64);
    notification_ptr_set_ntfnMsgIdentifier(ntfnPtr, badge);
}

// static inline void ntfn_set_active(notification_t *ntfnPtr, word_t badge)
// {
//     notification_ptr_set_state(ntfnPtr, NtfnState_Active);
//     notification_ptr_set_ntfnMsgIdentifier(ntfnPtr, badge);
// }