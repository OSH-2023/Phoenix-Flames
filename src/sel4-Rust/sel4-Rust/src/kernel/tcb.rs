#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::failures::exception::*;
use crate::failures::exception_t;
use crate::types::prio_t;

pub const L2_BITMAP_SIZE: usize = (256 + (1 << 6) - 1) / (1 << 6);
pub const wordRadix: u64 = 6;
pub const wordBits: u64 = 1 << 6;

static mut ksReadyQueuesL1Bitmap:[u64; 1];
static mut ksReadyQueuesL2Bitmap: [[u64; L2_BITMAP_SIZE]; 1];
static mut ksReadyQueues: [tcb_queue_t;256];
static mut ksCurThread:*mut tcb_t;

pub fn checkPrio(prio:prio_t, auth:*mut tcb_t) -> exception_t{
    let mut mcp:prio_t;
    unsafe{
        mcp = (*auth).tcbMCP;
    }
    //     assert(mcp <= seL4_MaxPrio);

    if(prio > mcp){
        current_syscall_error.type = seL4_RangeError;
        current_syscall_error.rangeErrorMin = seL4_MinPrio;
        current_syscall_error.rangeErrorMax = mcp;
        return EXCEPTION_SYSCALL_ERROR;
    }
    EXCEPTION_NONE
}

pub fn addToBitmap(cpu:word_t, dom:word_t, prio:word_t){
    let l1index:word_t = prio_to_l1index(prio);
    let l1index_inverted:word_t = invert_l1index(l1index);
    ksReadyQueuesL1Bitmap[dom as u64] |= BIT!(l1index);
    ksReadyQueuesL2Bitmap[dom][l1index_inverted] |= BIT!(prio & MASK!(wordRadix));
}

pub fn removeFromBitmap(cpu:word_t, dom:word_t, prio:word_t) {
    let l1index:word_t = prio_to_l1index(prio);
    let l1index_inverted = invert_l1index(l1index);
    ksReadyQueuesL2Bitmap[dom][l1index_inverted] &= !BIT!(prio & MASK!(wordRadix));
    if ksReadyQueuesL2Bitmap[dom][l1index_inverted]==0 {
        ksReadyQueuesL1Bitmap[dom] &= !BIT!(l1index);
    }
}

pub fn tcbSchedEnqueue(tcb:*mut tcb_t) {
    let mut tmp:u64;
    unsafe{
        tmp = thread_state_get_tcbQueued((*tcb).tcbState);
    }
    if tmp==0 {
        let mut dom:dom_t;
        let mut prio:prio_t;
        unsafe{
            dom = (*tcb).tcbDomain;
            prio = (*tcb).tcbPriority;
        }
        let idx:word_t = ready_queues_index(dom, prio);
        let queue:tcb_queue_t = ksReadyQueues[idx];

        if queue.end==0 as (*mut tcb_t){
            queue.end = tcb;
            addToBitmap(0, dom, prio);
        } else {
            unsafe{
                (*queue.head).tcbSchedPrev = tcb;
            }
        }
        unsafe{
            (*tcb).tcbSchedPrev = 0u64 as (*mut tcb);
            (*tcb).tcbSchedNext = queue.head;
        }
        queue.head = tcb;
        ksReadyQueues[idx] = queue;
        unsafe{
            thread_state_ptr_set_tcbQueued((*tcb).tcbState, 1);
        }
    }
}

pub fn tcbSchedAppend(tcb:*mut tcb_t){
    let mut tmp:u64;
    unsafe{
        tmp = thread_state_get_tcbQueued((*tcb).tcbState);
    }
    if tmp==0 {
        let mut dom:dom_t;
        let mut prio:prio_t;
        unsafe{
            dom = (*tcb).tcbDomain;
            prio = (*tcb).tcbPriority;
        }
        let idx:word_t = ready_queues_index(dom, prio);
        let queue:tcb_queue_t = ksReadyQueues[idx];
        if queue.head == 0 as (*mut tcb_t) {
            queue.head = tcb;
            addToBitmap(0, dom, prio);
        } else {
            (*queue.end).tcbSchedNext = tcb;
        }
        unsafe{
            (*tcb).tcbSchedPrev = queue.end;
            (*tcb).tcbSchedNext = 0 as (*mut tcb);
        }
        queue.end = tcb;
        ksReadyQueues[idx] = queue;
        unsafe{
            thread_state_ptr_set_tcbQueued((*tcb).tcbState, 1);
        }
    }
}

pub fn tcbSchedDequeue(tcb: *mut tcb_t){
    let mut tmp:u64;
    unsafe{
        tmp = thread_state_get_tcbQueued((*tcb).tcbState);
    }
    if tmp==0 {
        let mut dom:dom_t;
        let mut prio:prio_t;
        unsafe{
            dom = (*tcb).tcbDomain;
            prio = (*tcb).tcbPriority;
        }
        let idx:word_t = ready_queues_index(dom, prio);
        let queue:tcb_queue_t = ksReadyQueues[idx];
        unsafe{
            if (*tcb).tcbSchedPrev != 0 as (*mut tcb) {
                (*(*tcb).tcbSchedPrev).tcbSchedNext = (*tcb).tcbSchedNext;
            } else {
                    queue.head = (*tcb).tcbSchedNext;
                    if (*tcb).tcbSchedNext == 0 as (*mut tcb) {
                        removeFromBitmap(0, dom, prio);
                    }
                }
            }
        }
        ksReadyQueues[idx] = queue;
        unsafe{
            thread_state_ptr_set_tcbQueued((*tcb).tcbState, false);
        }
}

pub fn tcbEPAppend(tcb: *mut tcb_t, queue: tcb_queue_t) -> tcb_queue_t {
    if queue.head== 0 as (*mut tcb_t) {
        queue.head = tcb;
    } else {
        unsafe{
            (*queue.end).tcbEPNext = tcb;
        }
    }
    unsafe{
        (*tcb).tcbEPPrev = queue.end;
        (*tcb).tcbEPNext = 0 as (*mut tcb);
    }
    queue.end = tcb;
    queue
}

pub fn tcbEPDequeue(tcb:*mut tcb_t, queue:tcb_queue_t) -> tcb_queue_t{
    unsafe{
        if (*tcb).tcbEPPrev != 0 as (*mut tcb) {
            (*(*tcb).tcbEPPrev).tcbEPNext = (*tcb).tcbEPNext;
        } else {
            queue.head = (*tcb).tcbEPNext;
        }

        if (*tcb).tcbEPNext != 0 as (*mut tcb) {
            (*(*tcb).tcbEPNext).tcbEPPrev = (*tcb).tcbEPPrev;
        } else {
            queue.end = (*tcb).tcbEPPrev;
        }

        queue
    }
}

pub fn getExtraCPtr(bufferPtr:*mut word_t, i:word_t) {
    unsafe{
        bufferPtr[120 + 2 + i] as cptr_t;
    }
}

pub fn setExtraBadge(buffer:*mut word_t, badge:word_t, i:word_t){
    unsafe{
        bufferPtr[120 + 2 + i] = badge;
    }
}

pub fn setupCallerCap(sender:*mut tcb_t, receiver:*mut tcb_t, canGrant:u64){
    let mut replySlot:*mut cte_t;
    let mut callerSlot:*mut cte_t;
    let mut masterCap:cap_t;
    let mut callerCap:cap_t;

    setThreadState(sender, ThreadState_BlockedOnReply);
    replySlot = TCB_PTR_CTE_PTR(sender, tcbReply);
    unsafe{
        masterCap = (*replySlot).cap;
        callerCap = (*callerSlot).cap;
    }
    callerSlot = TCB_PTR_CTE_PTR(receiver, tcbCaller);
    cteInsert(cap_reply_cap_new(canGrant, false, TCB_REF(sender)),replySlot, callerSlot);
}

pub fn deleteCallerCap(reciever:*mut tcb_t) {
    let callerSlot:*mut cte_t = TCB_PTR_CTE_PTR(receiver, tcbCaller);
    cteDeleteOne(callerSlot);
}

static mut current_extra_caps:extra_caps_t;

pub fn lookupExtraCaps(thread:*mut tcb_t, bufferPtr:*mut word_t, info:seL4_MessageInfo_t) -> exception_t{
    let mut lu_ret:lookupSlot_raw_ret_t;
    let mut cptr:cptr_t;
    let mut i:word_t;
    let mut length:word_t;

    if bufferPtr == 0 as *mut word_t {
        current_extra_caps.excaprefs[0] = 0 as (*mut cte_t);
        return EXCEPTION_NONE;
    }

    length = seL4_MessageInfo_get_extraCaps(info);

    i = 0;
    while i<length {
        cptr = getExtraCPtr(bufferPtr, i);
        lu_ret = lookupSlot(thread,cptr);
        if lu_ret.status != EXCEPTION_NONE {
            current_fault = seL4_Fault_CapFault_new(cptr, false);
            return lu_ret.status;
        }
        current_extra_caps.excaprefs[i] = lu_ret.slot;

        i+=1;
    }

    if i<3 {
        current_extra_caps.excaprefs[i] = 0 as (*mut cte_t);
    }
    EXCEPTION_NONE
}

pub fn copyMRs(sender:*mut tcb_t, sendBuf:*mut word_t, reciever:*mut tcb_t, recvBuf:*mut word_t, n:word_t) -> word_t {
    let mut i:word_t = 0;
    while i<n && i<4 {
        setRegister(receiver, msgRegisters[i], getRegister(sender, msgRegisters[i]));

        i+=1;
    }

    if recvBuf == 0 as *mut word_t || sendBuf == 0 as *mut word_t {
        return i;
    }

    while i<n {
        unsafe{
            recvBuf[i+1] = sendBuf[i+1];
        }

        i+=1;
    }
    i
}

pub fn invokeSetTLSBase(thread:*mut tcb_t, tls_base:word_t) -> exception_t{
    setRegister(thread, TLS_BASE, tls_base);
    if thread == ksCurThread {
        rescheduleRequired();
    }
    EXCEPTION_NONE
}

pub fn decodeSetTLSBase(cap:cap_t, length:word_t, buffer:*mut word_t) -> exception_t {
    let mut tls_base:word_t;
    if length<1 {
        // userError("TCB SetTLSBase: Truncated message.");
        current_syscall_error.type = seL4_TruncatedMessage;
        return EXCEPTION_SYSCALL_ERROR;
    }
    tls_base = getSyscallArg(0, buffer);

    setThreadState(NODE_STATE(ksCurThread), ThreadState_Restart);
    invokeSetTLSBase(TCB_PTR(cap_thread_cap_get_capTCBPtr(cap)), tls_base)
}

pub fn decodeTCBInvocation(invLabel:word_t, length:word_t, cap:cap_t, slot:*mut cte_t, call:word_t, buffer:*word_t) -> exception_t {
    if invLabel == 
}


exception_t decodeTCBInvocation(word_t invLabel, word_t length, cap_t cap,
                                cte_t *slot, bool_t call, word_t *buffer)
{
    /* Stall the core if we are operating on a remote TCB that is currently running */
    SMP_COND_STATEMENT(remoteTCBStall(TCB_PTR(cap_thread_cap_get_capTCBPtr(cap)));)

    switch (invLabel) {
    case TCBReadRegisters:
        /* Second level of decoding */
        return decodeReadRegisters(cap, length, call, buffer);

    case TCBWriteRegisters:
        return decodeWriteRegisters(cap, length, buffer);

    case TCBCopyRegisters:
        return decodeCopyRegisters(cap, length, buffer);

    case TCBSuspend:
        /* Jump straight to the invoke */
        setThreadState(NODE_STATE(ksCurThread), ThreadState_Restart);
        return invokeTCB_Suspend(
                   TCB_PTR(cap_thread_cap_get_capTCBPtr(cap)));

    case TCBResume:
        setThreadState(NODE_STATE(ksCurThread), ThreadState_Restart);
        return invokeTCB_Resume(
                   TCB_PTR(cap_thread_cap_get_capTCBPtr(cap)));

    case TCBConfigure:
        return decodeTCBConfigure(cap, length, slot, buffer);

    case TCBSetPriority:
        return decodeSetPriority(cap, length, buffer);

    case TCBSetMCPriority:
        return decodeSetMCPriority(cap, length, buffer);

    case TCBSetSchedParams:
        return decodeSetSchedParams(cap, length, buffer);
    case TCBSetIPCBuffer:
        return decodeSetIPCBuffer(cap, length, slot, buffer);

    case TCBSetSpace:
        return decodeSetSpace(cap, length, slot, buffer);

    case TCBBindNotification:
        return decodeBindNotification(cap);

    case TCBUnbindNotification:
        return decodeUnbindNotification(cap);
    case TCBSetTLSBase:
        return decodeSetTLSBase(cap, length, buffer);

    default:
        /* Haskell: "throw IllegalOperation" */
        userError("TCB: Illegal operation.");
        current_syscall_error.type = seL4_IllegalOperation;
        return EXCEPTION_SYSCALL_ERROR;
    }
}