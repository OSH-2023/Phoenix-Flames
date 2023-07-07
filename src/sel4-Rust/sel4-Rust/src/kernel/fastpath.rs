// WZC 7/4 16:35
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::sync::UPSafeCell;
use lazy_static::lazy_static;
use crate::types::*;
use crate::object::*;
use core::intrinsics::{likely, unlikely};
// use crate::kernel::cspace;
// use crate::cnode::*;

lazy_static! {
    static ref ksCurThread: *mut tcb_t;
    static ref ksCurDomain: dom_t;
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
// 可能于fault中的内容重复
#[repr(C)]
pub enum seL4_Fault_tag {
    seL4_Fault_NullFault = 0,
    seL4_Fault_CapFault = 1,
    seL4_Fault_UnknownSyscall = 2,
    seL4_Fault_UserException = 3,
    seL4_Fault_VMFault = 5,
}
#[repr(C)]
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
    tcbCNodeEntries,
}
#[repr(C)]
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
    cap_io_port_control_cap = 31,
}
#[repr(C)]
pub enum endpoint_state {
    EPState_Idle = 0,
    EPState_Send = 1,
    EPState_Recv = 2
}
pub type endpoint_state_t = word_t;

pub enum domainConstants {
    // #define CONFIG_NUM_DOMAINS  1
    minDom = 0,
    maxDom = 1 - 1,
    numDomains = 1,
}

// 数据类型
#[repr(C)]
#[derive(Copy, Clone)]
pub struct seL4_MessageInfo {
    words: [u64; 1],
}
pub type seL4_MessageInfo_t = seL4_MessageInfo;

pub struct seL4_Fault {
    words:[u64;2],
}
pub type seL4_Fault_t = seL4_Fault;

pub struct endpoint {
    words: [u64; 2],
}
pub type endpoint_t = endpoint;

pub struct pml4e {
    words: [u64; 1],
}
pub type pml4e_t = pml4e;
pub type vspace_root_t = pml4e_t;

pub struct pde {
    words: [u64; 1],
}
pub type pde_t = pde;

pub struct thread_state {
    words: [u64; 3],
}
pub type thread_state_t = thread_state;

pub struct mdb_node {
    words: [u64; 2],
}
pub type mdb_node_t = mdb_node;

// 外调C原始函数
extern "C" {
    fn slowpath(syscall: word_t); // in kernel_all
    // 下面这一函数在keral_all中没有定义，在kernal_all_pp_prune中有定义
    fn messageInfoFromWord_raw(w:word_t) -> seL4_MessageInfo_t;
    // 这个函数没有找到定义
    fn seL4_MessageInfo_get_length(info: seL4_MessageInfo_t) -> word_t;
    // 在kernal.i中定义
    fn seL4_Fault_get_seL4_FaultType(seL4_Fault: seL4_Fault_t) -> u64;
    // 在kernal.i中
    fn cap_capType_equals(cap: cap_t, cap_type_tag: u64) -> bool_t;
    fn fastpath_mi_check(msgInfo: word_t) -> u64;
    fn lookup_fp(cap: cap_t,cptr: cptr_t) -> cap_t;
    fn cap_endpoint_cap_get_capCanSend(cap: cap_t) -> u64;
    fn cap_endpoint_cap_get_capEPPtr(cap: cap_t) -> u64;
    fn endpoint_ptr_get_epQueue_head(endpoint_ptr: *mut endpoint_t) -> u64;
    fn endpoint_ptr_get_state(endpoint_ptr: *mut endpoint_t) -> u64;
    fn cap_vtable_cap_get_vspace_root_fp(vtable_cap: cap_t) -> *mut vspace_root_t;
    fn isValidVTableRoot_fp(vspace_root_cap: cap_t) -> bool_t;
    fn cap_pml4_cap_get_capPML4MappedASID_fp(vtable_cap: cap_t) -> u64;
    fn isHighestPrio(dom: word_t, prio: prio_t) -> bool_t;
    fn cap_endpoint_cap_get_capCanGrant(cap: cap_t) -> u64;
    fn cap_endpoint_cap_get_capCanGrantReply(cap: cap_t) -> u64;
    fn endpoint_ptr_set_epQueue_head_np(ep_ptr: *mut endpoint_t, epQueue_head: word_t);
    fn endpoint_ptr_mset_epQueue_tail_state(ep_ptr: *mut endpoint_t, epQueue_tail: word_t,state: word_t);
    fn cap_endpoint_cap_get_capEPBadge(cap: cap_t) -> u64;
    fn thread_state_ptr_set_tsType_np(ts_ptr: *mut thread_state_t, tsType: word_t);
    fn thread_state_ptr_get_blockingIPCCanGrant(thread_state_ptr: *mut thread_state_t);
    fn cap_reply_cap_ptr_new_np(cap_ptr: *mut cap_t, capReplyCanGrant: word_t, 
                                capReplyMaster: word_t, capTCBPtr: word_t);
    fn mdb_node_ptr_set_mdbPrev_np(node_ptr: *mut mdb_node_t, mdbPrev: word_t);
    fn mdb_node_ptr_mset_mdbNext_mdbRevocable_mdbFirstBadged(node_ptr: *mut mdb_node_t, mdbNext: word_t,
                                                             mdbRevocable: word_t, mdbFirstBadged: word_t);
    fn fastpath_copy_mrs(length: word_t, src: *mut tcb_t, dest: *mut tcb_t);
    fn switchToThread_fp(thread: *mut tcb_t, vroot: *mut vspace_root_t, stored_hw_asid: pde_t);
    fn wordFromMessageInfo(mi: seL4_MessageInfo_t) -> word_t;
    fn seL4_MessageInfo_set_capsUnwrapped(seL4_MessageInfo: seL4_MessageInfo_t, v64: u64) -> seL4_MessageInfo_t;
    fn fastpath_restore(badge: word_t, msgInfo: word_t, cur_thread: *mut tcb_t);

    fn cap_endpoint_cap_get_capCanReceive(cap: cap_t) -> u64;
    fn fastpath_reply_cap_check(cap: cap_t) -> word_t;
    fn cap_reply_cap_get_capTCBPtr(cap: cap_t) -> u64;
    fn cap_pml4_cap_get_capPML4MappedASID(cap: cap_t) -> u64;
    fn thread_state_ptr_mset_blockingObject_tsType(ts_ptr: *mut thread_state_t, ep_ref: word_t, tsType: word_t);
    fn thread_state_ptr_set_blockingIPCCanGrant(thread_state_ptr: *mut thread_state_t, v64: u64);
    fn endpoint_ptr_get_epQueue_tail_fp(ep_ptr: endpoint_t) -> *mut tcb_t;
    fn mdb_node_get_mdbPrev(mdb_node: mdb_node_t) -> u64;
    fn cap_null_cap_new() -> cap_t;
    fn mdb_node_new(mdbNext: u64, mdbRevocable: u64, mdbFirstBadged: u64, mdbPrev: u64) -> mdb_node_t;
}






// funcs
#[no_mangle]
pub fn fastpath_call(
    cptr:       word_t,
    msgInfo:    word_t
){
    // global
    let mut use_ksCurThread = ksCurThread.exclusive_access();
    let mut use_ksCurDomain = ksCurDomain.exclusive_access();
    let use_seL4_Fault_tag: seL4_Fault_tag;
    let use_syscall: syscall;
    let use_tcb_cnode_index: tcb_cnode_index;
    let use_cap_tag: cap_tag;
    let use_endpoint_state: endpoint_state;
    let use_domainConstants: domainConstants;
    let use_thread_state: _thread_state;

    
    unsafe{
        // 获取消息内容与长度
        let info: seL4_MessageInfo_t = messageInfoFromWord_raw(msgInfo);
        let length: word_t = seL4_MessageInfo_get_length(info);
        let fault_type = seL4_Fault_get_seL4_FaultType(&(*(NODE_STATE(use_ksCurThread))).tcbFault);
        // 不可以有额外能力，长度不符合要求，没有保存的错误，否则转入slowpath
        let fault_type = seL4_Fault_get_seL4_FaultType(&(*(NODE_STATE(use_ksCurThread))).tcbFault);
        if unlikely(fastpath_mi_check(msgInfo) || 
                    fault_type != use_seL4_Fault_tag.seL4_Fault_NullFault as u64) {
            slowpath(use_syscall.SysCall);
        }
        // 查找能力
        let ep_cap = lookup_fp(&(*(TCB_PTR_CTE_PTR(NODE_STATE(use_ksCurThread), use_tcb_cnode_index.tcbCTable))).cap, cptr);
        // 是否是端点
        if unlikely(!cap_capType_equals(ep_cap, use_cap_tag.cap_endpoint_cap) ||
                    !cap_endpoint_cap_get_capCanSend(ep_cap)) {
            slowpath(use_syscall.SysCall);
        }
        // 获取端点地址
        let ep_ptr: *const endpoint_t = EP_PTR(cap_endpoint_cap_get_capEPPtr(ep_cap));
        // 获取目标线程tcb地址
        let dest: *const tcb_t = TCB_PTR(endpoint_ptr_get_epQueue_head(ep_ptr));
        // 检查等待接受的线程
        if unlikely(endpoint_ptr_get_state(ep_ptr) != use_endpoint_state.EPState_Recv) {
            slowpath(use_syscall.SysCall);
        }
        let dest = TCB_PTR(endpoint_ptr_get_epQueue_head(ep_ptr));
        // 获取目标线程的VTable
        let newVTable: *const cap_t = &(*(TCB_PTR_CTE_PTR(dest, use_tcb_cnode_index.tcbVTable))).cap;
        // 获取vspace根地址
        let cap_pd: *const vspace_root_t = cap_vtable_cap_get_vspace_root_fp(newVTable);
        // 确保VTable合法
        if unlikely(! isValidVTableRoot_fp(newVTable)) {
            slowpath(use_syscall.SysCall);
        }

        // x86 获取ASID
        let mut stored_hw_asid: pde_t;
        // 这里的数组引用是否正确？
        stored_hw_asid.words.0 = cap_pml4_cap_get_capPML4MappedASID_fp(use_tcb_cnode_index.newVTable);

        // let gcc optimise this out for 1 domain
        let dom: dom_t = if use_domainConstants.maxDom!=0 {use_ksCurDomain} else {0};
        // 保证现在只有空闲和低优先级线程在调度中
        if (unlikely(&(*(dest)).tcbPriority < NODE_STATE(&(*(use_ksCurThread)).tcbPriority) &&
                    !isHighestPrio(dom, &(*(dest)).tcbPriority))) {
            slowpath(use_syscall.SysCall);
        }
        // 保证端点有被授予的能力
        if unlikely(!cap_endpoint_cap_get_capCanGrant(ep_cap) &&
                        !cap_endpoint_cap_get_capCanGrantReply(ep_cap)) {
            slowpath(use_syscall.SysCall);
        }
        // 保证原始的调用者线程正在现在的域，可以被直接调度
        if unlikely(&(*(dest)).tcbDomain != use_ksCurDomain && 0 < use_domainConstants.maxDom) {
            slowpath(use_syscall.SysCall);
        }

        // ------------------------------------
        // 经过了前面的准备，这里可以开始进行IPC的实现
        // 目标线程出队
        endpoint_ptr_set_epQueue_head_np(ep_ptr, TCB_REF(&(*(dest)).tcbEPNext));
        if unlikely(&(*(dest)).tcbEPNext) {
            &(*(&(*(dest)).tcbEPNext)).tcbEPPrev = 0;
        } else {
            endpoint_ptr_mset_epQueue_tail_state(ep_ptr, 0, use_endpoint_state.EPState_Idle);
        }
        let bagde: word_t = cap_endpoint_cap_get_capEPBadge(&ep_cap);
        thread_state_ptr_set_tsType_np(NODE_STATE(&use_ksCurThread).tcbState,
                                        &use_thread_state.ThreadState_BlockedOnReply);

        // 获取发送者用于回复的能力插槽
        let replySlot: *const cte_t = TCB_PTR_CTE_PTR(NODE_STATE(use_ksCurThread), use_tcbReply);
        // 获取目标调用者的能力插槽
        let callerSlot: *const cte_t = TCB_PTR_CTE_PTR(dest, use_tcbCaller);

        // 把回复的能力插入其中
        let replyCanGrant:  word_t = thread_state_ptr_get_blockingIPCCanGrant((dest.tcbState));
        cap_reply_cap_ptr_new_np(callerSlot.cap, replyCanGrant, 0,
                                TCB_REF(NODE_STATE(use_ksCurThread)));
        mdb_node_ptr_set_mdbPrev_np(callerSlot.cteMDBNode, CTE_REF(replySlot));
        mdb_node_ptr_mset_mdbNext_mdbRevocable_mdbFirstBadged(
            replySlot.cteMDBNode, CTE_REF(callerSlot), 1, 1);

        fastpath_copy_mrs(length, NODE_STATE(use_ksCurThread), dest);

        // 目标线程运行
        thread_state_ptr_set_tsType_np(dest.tcbState,
                                    use_thread_state.ThreadState_Running);
        switchToThread_fp(dest, cap_pd, stored_hw_asid);

        msgInfo = wordFromMessageInfo(seL4_MessageInfo_set_capsUnwrapped(info, 0));
        
        let badge: word_t = 0;
        fastpath_restore(badge, msgInfo, NODE_STATE(use_ksCurThread));
    }

    // drop global variable
    drop(use_ksCurThread);
    drop(use_ksCurDomain);
}

#[no_mangle]
pub fn fastpath_reply_recv(
    cptr: word_t,
    msgInfo: word_t
){
    // global
    let mut use_ksCurThread = ksCurThread.exclusive_access();
    let mut use_ksCurDomain = ksCurDomain.exclusive_access();
    let use_seL4_Fault_tag: seL4_Fault_tag;
    let use_syscall: syscall;
    let use_tcb_cnode_index: tcb_cnode_index;
    let use_cap_tag: cap_tag;
    let use_endpoint_state: endpoint_state;
    let use_domainConstants: domainConstants;
    let use_thread_state: _thread_state;

    unsafe{
        // 预先的检查
        let info: seL4_MessageInfo_t = messageInfoFromWord_raw(msgInfo);
        let length: word_t = seL4_MessageInfo_get_length(info);
        let fault_type: word_t = seL4_Fault_get_seL4_FaultType(&(*(NODE_STATE(use_ksCurThread))).tcbFault);
        // 一番检查
        if unlikely(fastpath_mi_check(msgInfo) || fault_type != use_seL4_Fault_tag.seL4_Fault_NullFault as u64) {
            slowpath(use_syscall.SysReplyRecv);
        }
        let ep_cap: cap_t = lookup_fp(TCB_PTR_CTE_PTR(&(*(NODE_STATE(use_ksCurThread), use_tcb_cnode_index.tcbCTable))).cap,cptr);
        if unlikely(!cap_capType_equals(ep_cap, use_cap_tag.cap_endpoint_cap) ||
                    !cap_endpoint_cap_get_capCanReceive(ep_cap)) {
            slowpath(use_syscall.SysReplyRecv);
        }
        if (unlikely(&(*(NODE_STATE(use_ksCurThread))).tcbBoundNotification &&
                    notification_ptr_get_state(&(*(NODE_STATE(use_ksCurThread))).tcbBoundNotification) == NtfnState_Active)) {
            slowpath(use_syscall.SysReplyRecv);
        }
        let ep_ptr: *const endpoint_t = EP_PTR(cap_endpoint_cap_get_capEPPtr(ep_cap));
        
        if unlikely(endpoint_ptr_get_state(ep_ptr) == use_endpoint_state.EPState_Send) {
            slowpath(use_syscall.SysReplyRecv);
        }
        let callerSlot: *const cte_t = TCB_PTR_CTE_PTR(NODE_STATE(use_ksCurThread), use_tcb_cnode_index.tcbCaller);
        let callerCap: cap_t = callerSlot.cap;
        if unlikely(!fastpath_reply_cap_check(callerCap)) {
            slowpath(use_syscall.SysReplyRecv);
        }
        let caller: *const tcb_t = TCB_PTR(cap_reply_cap_get_capTCBPtr(callerCap));
        fault_type = seL4_Fault_get_seL4_FaultType(&caller.tcbFault);
        if unlikely(fault_type != use_seL4_Fault_tag.seL4_Fault_NullFault && fault_type != use_seL4_Fault_tag.seL4_Fault_VMFault) {
            slowpath(use_syscall.SysReplyRecv);
        }
        let newVTable: cap_t = &(*(TCB_PTR_CTE_PTR(caller, use_tcb_cnode_index.tcbVTable))).cap;
        let cap_pd: *const vspace_root_t = cap_vtable_cap_get_vspace_root_fp(newVTable);
        if unlikely(! isValidVTableRoot_fp(newVTable)) {
            slowpath(use_syscall.SysReplyRecv);
        }
        // x86
        let mut stored_hw_asid: pde_t;
        stored_hw_asid.words[0] = cap_pml4_cap_get_capPML4MappedASID(newVTable);
        let dom: dom_t = if use_domainConstants.maxDom!=0 {use_ksCurDomain} else {0};
        if (unlikely(!isHighestPrio(dom, *caller.tcbPriority))) {
            slowpath(use_syscall.SysReplyRecv);
        }
        if unlikely(*caller.tcbDomain != use_ksCurDomain && 0 < use_domainConstants.maxDom) {
            slowpath(use_syscall.SysReplyRecv);
        }

        // ------------------------------------

        thread_state_ptr_mset_blockingObject_tsType(
            NODE_STATE(use_ksCurThread).tcbState, ep_ptr as word_t, use_thread_state.ThreadState_BlockedOnReceive);

        thread_state_ptr_set_blockingIPCCanGrant(NODE_STATE(use_ksCurThread).tcbState,
                                                cap_endpoint_cap_get_capCanGrant(ep_cap));
        
        /* Place the thread in the endpoint queue */
        let endpointTail: *const tcb_t = endpoint_ptr_get_epQueue_tail_fp(ep_ptr);
        if likely(!endpointTail) {
            &(*(NODE_STATE(use_ksCurThread))).tcbEPPrev = NULL;
            &(*(NODE_STATE(use_ksCurThread))).tcbEPNext = NULL;

            /* Set head/tail of queue and endpoint state. */
            endpoint_ptr_set_epQueue_head_np(ep_ptr, TCB_REF(NODE_STATE(use_ksCurThread)));
            endpoint_ptr_mset_epQueue_tail_state(ep_ptr, TCB_REF(NODE_STATE(use_ksCurThread)),
                                                use_endpoint_state.EPState_Recv);
        } else {

            /* Append current thread onto the queue. */
            &(*(endpointTail)).tcbEPNext = NODE_STATE(use_ksCurThread);
            &(*(NODE_STATE(use_ksCurThread))).tcbEPPrev = endpointTail;
            &(*(NODE_STATE(use_ksCurThread))).tcbEPNext = NULL;

            /* Update tail of queue. */
            endpoint_ptr_mset_epQueue_tail_state(ep_ptr, TCB_REF(NODE_STATE(use_ksCurThread)), EPState_Recv);
        }

        /* Delete the reply cap. */
        mdb_node_ptr_mset_mdbNext_mdbRevocable_mdbFirstBadged(
            CTE_PTR(mdb_node_get_mdbPrev(&callerSlot.cteMDBNode)).cteMDBNode,
            0, 1, 1);
        callerSlot.cap = cap_null_cap_new();
        // #define nullMDBNode mdb_node_new(0, false, false, 0)
        callerSlot.cteMDBNode = mdb_node_new(0, false, false, 0);

        /* Replies don't have a badge. */
        let badge: word_t = 0;

        fastpath_copy_mrs(length, NODE_STATE(use_ksCurThread), caller);

        /* Dest thread is set Running, but not queued. */
        thread_state_ptr_set_tsType_np(caller.tcbState, use_thread_state.ThreadState_Running);
        switchToThread_fp(caller, cap_pd, stored_hw_asid);

        msgInfo = wordFromMessageInfo(seL4_MessageInfo_set_capsUnwrapped(info, 0));

        fastpath_restore(badge, msgInfo, NODE_STATE(use_ksCurThread));
    }
    // drop global variable
    drop(use_ksCurThread);
    drop(use_ksCurDomain);
}
