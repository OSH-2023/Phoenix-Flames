// WZC 7/4 16:35
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::sync::UPSafeCell;
use lazy_static::lazy_static;
use crate::machine::*;
use crate::model::*;
use crate::types::*;
use crate::failures::*;
use crate::object::*;
use crate::cnode::*;

/* 全局变量：
* ksCurThread, SysReplyRecv, tcbCaller, ksCurDomain, tcbReply, ThreadState_Running
*/
lazy_static! {
    static ref ksCurThread: *mut tcb_t;
    static ref tcbCaller: mut u64;
    static ref ksCurDomain: dom_t;
    static ref tcbReply: u64 = 2;
}

#[repr(C)]
pub enum _thread_state {
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

#[repr(C)]
pub enum syscall {
    SysCall = -1,
    SysReplyRecv = -2,
    SysSend = -3,
    SysNBSend = -4,
    SysRecv = -5,
    SysReply = -6,
    SysYield = -7,
    SysNBRecv = -8,
}


// funcs
#[no_mangle]
pub fn fastpath_call(
    cptr:       word_t,
    msgInfo:    word_t
){
    // global
    let mut use_ksCurThread = ksCurThread.exclusive_access();
    let mut use_tcbCaller = tcbCaller.exclusive_access();
    let mut use_ksCurDomain = ksCurDomain.exclusive_access();
    let mut use_tcbReply = tcbReply.exclusive_access();

    // 获取消息内容与长度
    let info: MessageInfo_t = messageInfoFromWord_raw(msgInfo);
    let length: word_t = seL4_MessageInfo_get_length(info);
    unsafe{
        let fault_type = seL4_Fault_get_seL4_FaultType(&(*(NODE_STATE(ksCurThread))).tcbFault);
    }
    // 不可以有额外能力，长度不符合要求，没有保存的错误，否则转入slowpath
    if unlikely(fastpath_mi_check(msgInfo) || 
                fault_type != seL4_Fault_NullFault) {
        slowpath(SysCall);
    }
    // 查找能力
    unsafe{ep_cap = lookup_fp(&(*(TCB_PTR_CTE_PTR(NODE_STATE(ksCurThread), tcbCTable))).cap, cptr);}
    // 是否是端点
    if unlikely(!cap_capType_equals(ep_cap, cap_endpoint_cap) ||
                !cap_endpoint_cap_get_capCanSend(ep_cap)) {
        slowpath(SysCall);
    }
    unsafe{
        // 获取端点地址
        let ep_ptr: *const endpoint_t = EP_PTR(cap_endpoint_cap_get_capEPPtr(ep_cap));
        // 获取目标线程tcb地址
        let dest: *const tcb_t = TCB_PTR(endpoint_ptr_get_epQueue_head(ep_ptr));
    }
    // 检查等待接受的线程
    if unlikely(endpoint_ptr_get_state(ep_ptr) != EPState_Recv) {
        slowpath(SysCall);
    }

    unsafe{
        // 获取目标线程的VTable
        let newVTable: *const cap_t = &(*(TCB_PTR_CTE_PTR(dest, tcbVTable))).cap;
        // 获取vspace根地址
        let cap_pd: *const vspace_root_t = cap_vtable_cap_get_vspace_root_fp(newVTable);
    }
    // 确保VTable合法
    if unlikely(! isValidVTableRoot_fp(newVTable)) {
        slowpath(SysCall);
    }

    // x86 获取ASID
    let mut stored_hw_asid: pde_t;
    // 这里的数组引用是否正确？
    stored_hw_asid.words.0 = cap_pml4_cap_get_capPML4MappedASID_fp(newVTable);

    // let gcc optimise this out for 1 domain
    let dom: dom_t = if axDom!=0 {ksCurDomain} else {0};
    // 保证现在只有空闲和低优先级线程在调度中
    unsafe{
        if (unlikely(&(*(dest)).tcbPriority < NODE_STATE(&(*(ksCurThread)).tcbPriority) &&
                    !isHighestPrio(dom, &(*(dest)).tcbPriority))) {
            slowpath(SysCall);
        }
    }
    // 保证端点有被授予的能力
    if (unlikely(!cap_endpoint_cap_get_capCanGrant(ep_cap) &&
                 !cap_endpoint_cap_get_capCanGrantReply(ep_cap))) {
        slowpath(SysCall);
    }
    // 保证原始的调用者线程正在现在的域，可以被直接调度
    unsafe{
        if unlikely(&(*(dest)).tcbDomain != ksCurDomain && 0 < maxDom) {
            slowpath(SysCall);
        }
    }

    // ------------------------------------
    // 经过了前面的准备，这里可以开始进行IPC的实现
    // 目标线程出队
    unsafe{
        endpoint_ptr_set_epQueue_head_np(ep_ptr, TCB_REF(&(*(dest)).tcbEPNext));
        if (unlikely(&(*(dest)).tcbEPNext)) {
            &(*(&(*(dest)).tcbEPNext)).tcbEPPrev = NULL;
        } else {
            endpoint_ptr_mset_epQueue_tail_state(ep_ptr, 0, EPState_Idle);
        }
    }
    let bagde: word_t = cap_endpoint_cap_get_capEPBadge(&ep_cap);
    thread_state_ptr_set_tsType_np(NODE_STATE(&ksCurThread).tcbState,
                                   &ThreadState_BlockedOnReply);
    
    unsafe{
        // 获取发送者用于回复的能力插槽
        let replySlot: *const cte_t = TCB_PTR_CTE_PTR(NODE_STATE(ksCurThread), tcbReply);
        // 获取目标调用者的能力插槽
        let callerSlot: *const cte_t = TCB_PTR_CTE_PTR(dest, tcbCaller);
    }
    // 把回复的能力插入其中
    let replyCanGrant:  word_t = thread_state_ptr_get_blockingIPCCanGrant((dest.tcbState));
    cap_reply_cap_ptr_new_np(callerSlot.cap, replyCanGrant, 0,
                            TCB_REF(NODE_STATE(ksCurThread)));
    mdb_node_ptr_set_mdbPrev_np(callerSlot.cteMDBNode, CTE_REF(replySlot));
    mdb_node_ptr_mset_mdbNext_mdbRevocable_mdbFirstBadged(
        replySlot.cteMDBNode, CTE_REF(callerSlot), 1, 1);

    fastpath_copy_mrs(length, NODE_STATE(ksCurThread), dest);

    // 目标线程运行
    thread_state_ptr_set_tsType_np(dest.tcbState,
                                ThreadState_Running);
    switchToThread_fp(dest, cap_pd, stored_hw_asid);

    msgInfo = wordFromMessageInfo(seL4_MessageInfo_set_capsUnwrapped(info, 0));

    fastpath_restore(badge, msgInfo, NODE_STATE(ksCurThread));

    // drop global variable
    drop(use_ksCurThread);
    drop(use_tcbCaller);
    drop(use_ksCurDomain);
    drop(use_tcbReply);
}

#[no_mangle]
pub fn fastpath_rely_recv(
    cptr: word_t,
    msgInfo: word_t
){
    // global
    let mut use_ksCurThread = ksCurThread.exclusive_access();
    let mut use_tcbCaller = tcbCaller.exclusive_access();
    let mut use_ksCurDomain = ksCurDomain.exclusive_access();
    let mut use_tcbReply = tcbReply.exclusive_access();

    // 预先的检查
    let info: seL4_MessageInfo_t = messageInfoFromWord_raw(msgInfo);
    let length: word_t = seL4_MessageInfo_get_length(info);
    unsafe{let fault_type: word_t = seL4_Fault_get_seL4_FaultType(&(*(NODE_STATE(ksCurThread))).tcbFault);}
    // 一番检查
    if (unlikely(fastpath_mi_check(msgInfo) ||
                 fault_type != seL4_Fault_NullFault)) {
        slowpath(SysReplyRecv);
    }
    unsafe{let ep_cap: cap_t = lookup_fp(TCB_PTR_CTE_PTR(&(*(NODE_STATE(ksCurThread), tcbCTable))).cap,cptr);}
    if unlikely(!cap_capType_equals(ep_cap, cap_endpoint_cap) ||
                 !cap_endpoint_cap_get_capCanReceive(ep_cap)) {
        slowpath(SysReplyRecv);
    }
    unsafe{
        if (unlikely(&(*(NODE_STATE(ksCurThread))).tcbBoundNotification &&
                    notification_ptr_get_state(&(*(NODE_STATE(ksCurThread))).tcbBoundNotification) == NtfnState_Active)) {
            slowpath(SysReplyRecv);
        }
        let ep_ptr: *const endpoint_t = EP_PTR(cap_endpoint_cap_get_capEPPtr(ep_cap));
    }
    if unlikely(endpoint_ptr_get_state(ep_ptr) == EPState_Send) {
        slowpath(SysReplyRecv);
    }
    unsafe{let callerSlot: *const cte_t = TCB_PTR_CTE_PTR(NODE_STATE(ksCurThread), tcbCaller);}
    let callerCap: cap_t = callerSlot.cap;
    if unlikely(!fastpath_reply_cap_check(callerCap)) {
        slowpath(SysReplyRecv);
    }
    unsafe{let caller: *const tcb_t = TCB_PTR(cap_reply_cap_get_capTCBPtr(callerCap));}
    fault_type = seL4_Fault_get_seL4_FaultType(&caller.tcbFault);
    if unlikely(fault_type != seL4_Fault_NullFault && fault_type != seL4_Fault_VMFault) {
        slowpath(SysReplyRecv);
    }
    unsafe{let newVTable: cap_t = &(*(TCB_PTR_CTE_PTR(caller, tcbVTable))).cap;}
    unsafe{let cap_pd: *const vspace_root_t = cap_vtable_cap_get_vspace_root_fp(newVTable);}
    if unlikely(! isValidVTableRoot_fp(newVTable)) {
        slowpath(SysReplyRecv);
    }
    // x86
    let mut stored_hw_asid: pde_t;
    stored_hw_asid.words[0] = cap_pml4_cap_get_capPML4MappedASID(newVTable);
    let dom: dom_t = if maxDom!=0 {ksCurDomain} else {0};
    unsafe
    {
        if (unlikely(!isHighestPrio(dom, *caller.tcbPriority))) {
            slowpath(SysReplyRecv);
        }
        if unlikely(*caller.tcbDomain != ksCurDomain && 0 < maxDom) {
            slowpath(SysReplyRecv);
        }
    }

    // ------------------------------------

    thread_state_ptr_mset_blockingObject_tsType(
        NODE_STATE(ksCurThread).tcbState, ep_ptr as word_t, ThreadState_BlockedOnReceive);

    thread_state_ptr_set_blockingIPCCanGrant(NODE_STATE(ksCurThread).tcbState,
                                             cap_endpoint_cap_get_capCanGrant(ep_cap));
    
    /* Place the thread in the endpoint queue */
    unsafe{let endpointTail: *const tcb_t = endpoint_ptr_get_epQueue_tail_fp(ep_ptr);}
    unsafe{
        if likely(!endpointTail) {
            &(*(NODE_STATE(ksCurThread))).tcbEPPrev = NULL;
            &(*(NODE_STATE(ksCurThread))).tcbEPNext = NULL;

            /* Set head/tail of queue and endpoint state. */
            endpoint_ptr_set_epQueue_head_np(ep_ptr, TCB_REF(NODE_STATE(ksCurThread)));
            endpoint_ptr_mset_epQueue_tail_state(ep_ptr, TCB_REF(NODE_STATE(ksCurThread)),
                                                EPState_Recv);
        } else {

            /* Append current thread onto the queue. */
            &(*(endpointTail)).tcbEPNext = NODE_STATE(ksCurThread);
            &(*(NODE_STATE(ksCurThread))).tcbEPPrev = endpointTail;
            &(*(NODE_STATE(ksCurThread))).tcbEPNext = NULL;

            /* Update tail of queue. */
            endpoint_ptr_mset_epQueue_tail_state(ep_ptr, TCB_REF(NODE_STATE(ksCurThread)), EPState_Recv);
        }
    }

    /* Delete the reply cap. */
    mdb_node_ptr_mset_mdbNext_mdbRevocable_mdbFirstBadged(
        CTE_PTR(mdb_node_get_mdbPrev(&callerSlot.cteMDBNode)).cteMDBNode,
        0, 1, 1);
    &callerSlot.cap = cap_null_cap_new();
    &callerSlot.cteMDBNode = nullMDBNode;

    /* Replies don't have a badge. */
    let badge: word_t = 0;

    fastpath_copy_mrs(length, NODE_STATE(ksCurThread), caller);

    /* Dest thread is set Running, but not queued. */
    thread_state_ptr_set_tsType_np(caller.tcbState, ThreadState_Running);
    switchToThread_fp(caller, cap_pd, stored_hw_asid);

    msgInfo = wordFromMessageInfo(seL4_MessageInfo_set_capsUnwrapped(info, 0));

    fastpath_restore(badge, msgInfo, NODE_STATE(ksCurThread));

    // drop global variable
    drop(use_ksCurThread);
    drop(use_tcbCaller);
    drop(use_ksCurDomain);
    drop(use_tcbReply);
}
