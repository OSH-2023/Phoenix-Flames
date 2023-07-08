#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::failures::exception::*;
use crate::failures::exception_t;
use crate::types::prio_t;
use crate::failures::*;
use crate::types::word_t;
use crate::object::*;
use crate::failures::seL4_Error::*;
use crate::object::priorityConstants::*;
use crate::types::basic_types::{cptr_t, dom_t, };
use crate::object::_thread_state::*;
use crate::object::tcb_cnode_index::*;
use crate::kernel::tcb::CopyRegistersFlags::*;
use crate::kernel::thread::*;
use crate::kernel::cspace::lookupSlot_raw_ret_t;
use crate::types::compound_types::*;
use crate::cnode::*;
use crate::kernel::fastpath::seL4_MessageInfo_t;
use crate::kernel::tcb::invocation_label::*;
use crate::object::cap_tag::cap_thread_cap;
use crate::inlines::current_fault;

pub const L2_BITMAP_SIZE: usize = (256 + (1 << 6) - 1) / (1 << 6);
pub const wordRadix: u64 = 6;
pub const wordBits: u64 = 1 << 6;

extern "C" {
    pub fn prio_to_l1index(prio:word_t) -> word_t;
    pub fn thread_state_get_tcbQueued(thread_state:thread_state_t) -> u64;
    pub fn cap_reply_cap_new(capReplyCanGrant:u64, capReplyMaster:u64, capTCBPtr:u64) -> cap_t;
    pub fn cteInsert(newCap:cap_t, srcSlot:*mut cte_t, desSlot:*mut cte_t);
    pub fn seL4_MessageInfo_get_extraCaps(seL4_MessageInfo:seL4_MessageInfo_t) -> u64;
    pub fn lookupSlot(thread:*mut tcb_t, capptr:cptr_t) -> lookupSlot_raw_ret_t;
    pub fn cap_thread_cap_get_capTCBPtr(cap:cap_t) -> u64;
    pub fn seL4_Fault_CapFault_new(address:u64, inReceivePhase:u64) -> seL4_Fault_t;
    pub fn getSyscallArg(i:word_t, ipc_buffer:*mut word_t) -> word_t;
    pub fn Arch_decodeTransfer(flags:word_t) -> word_t;
    pub fn invokeTCB_CopyRegisters(dest:*mut tcb_t, src:*mut tcb_t, suspendSource:bool_t, resumeTarget:bool_t, transferFrame:bool_t, transferInteger:bool_t, transferArch:word_t) -> exception_t;
    pub static mut msgRegisters:[u8;2];
}

pub struct tcb_queue {
    pub head:*mut tcb_t,
    pub end:*mut tcb_t,
}
pub type tcb_queue_t = tcb_queue;

macro_rules! MASK {
    ($x:expr) => {
        (1u64 << ($x)) - 1u64
    };
}

macro_rules! BIT {
    ($x:expr) => {
        (1u64 << ($x))
    };
}

extern "C" {
    static mut ksReadyQueuesL1Bitmap:[u64; 1];
    static mut ksReadyQueuesL2Bitmap: [[u64; L2_BITMAP_SIZE]; 1];
    static mut ksReadyQueues: [tcb_queue_t;256];
    static mut ksCurThread:*mut tcb_t;
    static mut current_extra_caps:extra_caps_t;
    static mut current_syscall_error:syscall_error_t;
}

pub fn checkPrio(prio:prio_t, auth:*mut tcb_t) -> exception_t{
    let mut mcp:prio_t;
    unsafe{
        mcp = (*auth).tcbMCP;
    }
    //     assert(mcp <= seL4_MaxPrio);
    unsafe{
        if(prio > mcp){
            current_syscall_error.error_type = seL4_RangeError as u64;
            current_syscall_error.rangeErrorMin = seL4_MinPrio as u64;
            current_syscall_error.rangeErrorMax = mcp;
            return EXCEPTION_SYSCALL_ERROR as u64;
        }
    }
    EXCEPTION_NONE as u64
}

pub fn addToBitmap(cpu:word_t, dom:word_t, prio:word_t){
    let mut l1index:word_t;
    unsafe{
        l1index = prio_to_l1index(prio);
    }
    let l1index_inverted:word_t = invert_l1index(l1index);
    unsafe{
        ksReadyQueuesL1Bitmap[dom as usize] |= BIT!(l1index);
        ksReadyQueuesL2Bitmap[dom as usize][l1index_inverted as usize] |= BIT!(prio & MASK!(wordRadix));
    }
}

pub fn removeFromBitmap(cpu:word_t, dom:word_t, prio:word_t) {
    let mut l1index:word_t;
    unsafe{
        l1index = prio_to_l1index(prio);
    }
    let l1index_inverted = invert_l1index(l1index);
    unsafe{
        ksReadyQueuesL2Bitmap[dom as usize][l1index_inverted as usize] &= !BIT!(prio & MASK!(wordRadix));
        if ksReadyQueuesL2Bitmap[dom as usize][l1index_inverted as usize]==0 {
            ksReadyQueuesL1Bitmap[dom as usize] &= !BIT!(l1index);
        }
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
            let idx:word_t = ready_queues_index(dom, prio);
            let mut queue:tcb_queue_t = ksReadyQueues[idx as usize];

            if queue.end==0 as (*mut tcb_t){
                queue.end = tcb;
                addToBitmap(0, dom, prio);
            } else {
                (*queue.head).tcbSchedPrev = tcb;
            }
            (*tcb).tcbSchedPrev = 0u64 as (*mut tcb);
            (*tcb).tcbSchedNext = queue.head;
            queue.head = tcb;
            ksReadyQueues[idx as usize] = queue;
            unsafe{
                thread_state_ptr_set_tcbQueued(&mut (*tcb).tcbState as *mut thread_state, 1);
            }
        }
    }
}

pub fn tcbSchedAppend(mut tcb:*mut tcb_t){
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
            let idx:word_t = ready_queues_index(dom, prio);
            let mut queue:tcb_queue_t = ksReadyQueues[idx as usize];
            if queue.head == 0 as (*mut tcb_t) {
                queue.head = tcb;
                addToBitmap(0, dom, prio);
            } else {
                (*queue.end).tcbSchedNext = tcb;
            }
                (*tcb).tcbSchedPrev = queue.end;
                (*tcb).tcbSchedNext = 0 as (*mut tcb);
            queue.end = tcb;
            ksReadyQueues[idx as usize] = queue;
            thread_state_ptr_set_tcbQueued(&mut (*tcb).tcbState as *mut thread_state, 1);
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
        unsafe{
            let mut queue:tcb_queue_t = ksReadyQueues[idx as usize];
            if (*tcb).tcbSchedPrev != 0 as (*mut tcb) {
                (*(*tcb).tcbSchedPrev).tcbSchedNext = (*tcb).tcbSchedNext;
            } else {
                queue.head = (*tcb).tcbSchedNext;
                if (*tcb).tcbSchedNext == 0 as (*mut tcb) {
                    removeFromBitmap(0, dom, prio);
                }
            }
            ksReadyQueues[idx as usize] = queue;
            thread_state_ptr_set_tcbQueued(&mut (*tcb).tcbState as *mut thread_state, 0);
        }
    }
}

pub fn tcbEPAppend(tcb: *mut tcb_t, mut queue: tcb_queue_t) -> tcb_queue_t {
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

pub fn tcbEPDequeue(tcb:*mut tcb_t, mut queue:tcb_queue_t) -> tcb_queue_t{
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

pub fn getExtraCPtr(bufferPtr:*mut word_t, i:word_t) -> word_t{
    unsafe{
        *bufferPtr.offset((120 + 2 + i) as isize)
    }
}

pub fn setExtraBadge(bufferPtr:*mut word_t, badge:word_t, i:word_t){
    unsafe{
        *bufferPtr.offset((120 + 2 + i) as isize) = badge;
    }
}

pub fn setupCallerCap(sender:*mut tcb_t, receiver:*mut tcb_t, canGrant:u64){
    let mut replySlot:*mut cte_t;
    let mut callerSlot:*mut cte_t;
    let mut masterCap:cap_t;
    let mut callerCap:cap_t;
    unsafe{
        setThreadState(sender, ThreadState_BlockedOnReply as u64);
    }
    replySlot = TCB_PTR_CTE_PTR(sender, tcbReply as u64);
    unsafe{
        masterCap = (*replySlot).cap;
        callerCap = (*callerSlot).cap;
    }
    callerSlot = TCB_PTR_CTE_PTR(receiver, tcbCaller as u64);
    unsafe{
        cteInsert(cap_reply_cap_new(canGrant, 0, sender as u64),replySlot, callerSlot);
    }
}

pub fn deleteCallerCap(receiver:*mut tcb_t) {
    let callerSlot:*mut cte_t = TCB_PTR_CTE_PTR(receiver, tcbCaller as u64);
    cteDeleteOne(callerSlot);
}

pub fn lookupExtraCaps(thread:*mut tcb_t, bufferPtr:*mut word_t, info:seL4_MessageInfo_t) -> exception_t{
    let mut lu_ret:lookupSlot_raw_ret_t;
    let mut cptr:cptr_t;
    let mut i:word_t;
    let mut length:word_t;

    if bufferPtr == 0 as *mut word_t {
        unsafe{
            current_extra_caps.excaprefs[0] = 0 as (*mut cte_t);
        }
        return EXCEPTION_NONE as u64;
    }
    unsafe{
        length = seL4_MessageInfo_get_extraCaps(info);
    }

    i = 0;
    while i<length {
        cptr = getExtraCPtr(bufferPtr, i);
        unsafe{
            lu_ret = lookupSlot(thread,cptr);
        }
        if lu_ret.status != EXCEPTION_NONE as u64 {
            unsafe{
                current_fault = seL4_Fault_CapFault_new(cptr, 0);
            }
            return lu_ret.status;
        }
        unsafe{
            current_extra_caps.excaprefs[i as usize] = lu_ret.slot;
        }

        i+=1;
    }

    if i<3 {
        unsafe{
            current_extra_caps.excaprefs[i as usize] = 0 as (*mut cte_t);
        }
    }
    EXCEPTION_NONE as u64
}

pub fn copyMRs(sender:*mut tcb_t, sendBuf:*mut word_t, reciever:*mut tcb_t, recvBuf:*mut word_t, n:word_t) -> word_t {
    let mut i:word_t = 0;
    while i<n && i<4 {
        unsafe{
            setRegister(reciever, msgRegisters[i as usize] as u32, getRegister(sender, msgRegisters[i as usize] as u32));
        }

        i+=1;
    }

    if recvBuf == 0 as *mut word_t || sendBuf == 0 as *mut word_t {
        return i;
    }

    while i<n {
        unsafe{
            *recvBuf.offset((i+1) as isize) = *sendBuf.offset((i+1) as isize);
        }

        i+=1;
    }
    i
}

pub unsafe fn invokeSetTLSBase(thread:*mut tcb_t, tls_base:word_t) -> exception_t{
    setRegister(thread, 3, tls_base);
    if thread == ksCurThread {
        rescheduleRequired();
    }
    EXCEPTION_NONE as u64
}

pub unsafe fn decodeSetTLSBase(cap:cap_t, length:word_t, buffer:*mut word_t) -> exception_t {
    let mut tls_base:word_t;
    if length<1 {
        // userError("TCB SetTLSBase: Truncated message.");
        current_syscall_error.error_type = seL4_TruncatedMessage as u64;
        return EXCEPTION_SYSCALL_ERROR as u64;
    }
    unsafe{
        tls_base = getSyscallArg(0, buffer);
    }

    setThreadState(ksCurThread, ThreadState_Restart as u64);
    invokeSetTLSBase(cap_thread_cap_get_capTCBPtr(cap) as *mut tcb_t, tls_base)
}

// pub unsafe fn decodeTCBInvocation(invLabel:word_t, length:word_t, cap:cap_t, slot:*mut cte_t, call:word_t, buffer:*mut word_t) -> exception_t {
//     if invLabel == TCBReadRegisters{
//         return decodeReadRegisters(cap, length, call, buffer);
//     } else
//     if invLabel == TCBWriteRegisters{
//         return decodeWriteRegisters(cap, length, buffer);
//     } else
//     if invLabel == TCBCopyRegisters{
//         return decodeCopyRegisters(cap, length, buffer);
//     } else
//     if invLabel == TCBSuspend{
//         unsafe{
//             setThreadState(ksCurThread, ThreadState_Restart);
//         }
//         return invokeTCB_Suspend(cap_thread_cap_get_capTCBPtr(cap) as *mut tcb_t);
//     } else
//     if invLabel == TCBResume{
//         unsafe{
//             setThreadState(ksCurThread, ThreadState_Restart);
//         }
//         return invokeTCB_Suspend(cap_thread_cap_get_capTCBPtr(cap) as *mut tcb_t);
//     } else
//     if invLabel == TCBConfigure{
//         return decodeTCBConfigure(cap, length, slot, buffer);
//     } else
//     if invLabel == TCBSetPriority{
//         return decodeSetPriority(cap, length, buffer);
//     } else
//     if invLabel == TCBSetMCPriority{
//         return decodeSetMCPriority(cap, length, buffer);
//     } else
//     if invLabel == TCBSetSchedParams{
//         return decodeSetSchedParams(cap, length, buffer);
//     } else
//     if invLabel == TCBSetIPCBuffer{
//         return decodeSetIPCBuffer(cap, length, slot, buffer);
//     } else
//     if invLabel == TCBSetSpace{
//         return decodeSetSpace(cap, length, slot, buffer);
//     } else
//     if invLabel == TCBBindNotification{
//         return decodeBindNotification(cap);
//     } else
//     if invLabel == TCBUnbindNotification{
//         return decodeUnbindNotification(cap);
//     } else
//     if invLabel == TCBSetTLSBase{
//         unsafe{
//             return decodeSetTLSBase(cap, length, buffer);
//         }
//     } else {
//         // println!("TCB: Illegal operation.");
//         unsafe{
//             current_syscall_error.error_type = seL4_IllegalOperation;
//         }
//         return EXCEPTION_SYSCALL_ERROR;
//     }
//     EXCEPTION_SYSCALL_ERROR
// }

pub enum CopyRegistersFlags {
    CopyRegisters_suspendSource = 0,
    CopyRegisters_resumeTarget = 1,
    CopyRegisters_transferFrame = 2,
    CopyRegisters_transferInteger = 3
}

pub fn decodeCopyRegisters(cap:cap_t, length:word_t, buffer:*mut word_t) -> exception_t {
    let mut transferArch:word_t;
    let mut srcTCB:*mut tcb_t;
    let mut source_cap:cap_t;
    let mut flags:word_t;
    
    unsafe{
        if length<1 || current_extra_caps.excaprefs[0] == core::ptr::null_mut(){
            // println!("TCB CopyRegisters: Truncated message.");
            current_syscall_error.error_type = seL4_TruncatedMessage as u64;
            return EXCEPTION_SYSCALL_ERROR as u64;
        }
    }
    unsafe{
        flags = getSyscallArg(0, buffer);
        transferArch = Arch_decodeTransfer(flags >> 8);
        source_cap = (*current_extra_caps.excaprefs[0]).cap;
    }

    if cap_get_capType(source_cap) == cap_thread_cap as u64{
        unsafe{
            srcTCB = cap_thread_cap_get_capTCBPtr(source_cap) as *mut tcb_t;
        }
    } else {
        // println!("TCB CopyRegisters: Invalid source TCB.");
        unsafe{
            current_syscall_error.error_type = seL4_TruncatedMessage as u64;
        }
        return EXCEPTION_SYSCALL_ERROR as u64;
    }
    unsafe{
        setThreadState(ksCurThread, ThreadState_Restart as u64);
    }
    unsafe{
        invokeTCB_CopyRegisters(cap_thread_cap_get_capTCBPtr(cap) as *mut tcb_t, srcTCB, flags & (1u64<<(CopyRegisters_suspendSource as u64)), flags & (1u64<<(CopyRegisters_resumeTarget as u64)),
            flags & (1u64<<(CopyRegisters_transferFrame as u64)),
            flags & (1u64<<(CopyRegisters_transferInteger as u64)),
            transferArch)
    }
}

//invocation.h

pub enum invocation_label {
    InvalidInvocation,
    UntypedRetype,
    TCBReadRegisters,
    TCBWriteRegisters,
    TCBCopyRegisters,
    TCBConfigure,
    TCBSetPriority,
    TCBSetMCPriority,
    TCBSetSchedParams,
    TCBSetIPCBuffer,
    TCBSetSpace,
    TCBSuspend,
    TCBResume,
    TCBBindNotification,
    TCBUnbindNotification,
    TCBSetTLSBase,
    CNodeRevoke,
    CNodeDelete,
    CNodeCancelBadgedSends,
    CNodeCopy,
    CNodeMint,
    CNodeMove,
    CNodeMutate,
    CNodeRotate,
    CNodeSaveCaller,
    IRQIssueIRQHandler,
    IRQAckIRQ,
    IRQSetIRQHandler,
    IRQClearIRQHandler,
    DomainSetSet,
    nInvocationLabels
}
// not in this file, but for convenience we add it here