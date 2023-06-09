use crate::object::_thread_state::ThreadState_IdleThreadState;
use crate::object::_thread_state::ThreadState_Running;
use crate::object::tcb_t;
use crate::object::*;
use crate::types::prio_t;
use crate::types::word_t;
use crate::kernel::notification::cteDeleteOne;

extern "C" {
    fn setupReplyMaster(thread:*mut tcb);
    fn getReceiveSlots(thread:*mut tcb_t, buffer:*mut u64)->*mut cte_t;
}


#[inline]
pub fn thread_state_get_tsType(thread_state: *const thread_state_t) -> u64 {
    unsafe { (*thread_state).words[0] & 0xfu64 }
}

#[derive(Clone,Copy)]
#[repr(C)]
pub struct dschedule {
    pub domain: dom_t,
    pub length: word_t,
}
struct DeriveCapRet {
    status: exception_t,
    cap: cap_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct extra_caps {
    pub excaprefs: [cte_ptr_t; seL4_MsgMaxExtraCaps],
}
pub type cte_ptr_t = *mut cte_t;
pub type extra_caps_t = extra_caps;
type dom_t = word_t;
extern "C" {
    fn Arch_configureIdleThread(tcb: *mut tcb_t);
    static mut ksCurThread: *mut tcb_t;
    fn getRestartPC(thread: *mut tcb_t) -> word_t;
    fn setNextPC(thread: *mut tcb_t, v: word_t);
    fn Arch_activateIdleThread(tcb: *mut tcb_t);
    fn lookupIPCBuffer(isReceiver: bool_t, thread: *mut tcb_t) -> word_t;
    fn handleFaultReply(receiver: *mut tcb_t, sender: *mut tcb_t) -> bool_t;
    fn seL4_MessageInfo_get_length(info: seL4_MessageInfo_t) -> word_t;
    static mut current_extra_caps: extra_caps_t;
    fn setMRs_fault(
        sender: *mut tcb_t,
        receiver: *mut tcb_t,
        receiveIPCBuffer: *mut word_t,
    ) -> word_t;
    static mut ksDomScheduleIdx: word_t;
    static ksDomScheduleLength: word_t;
    static mut ksWorkUnitsCompleted: word_t;
    static ksDomSchedule: [dschedule; 1];
    static mut ksCurDomain: dom_t;
    static mut ksDomainTime: word_t;
    static mut ksSchedulerAction: *mut tcb_t;
    static mut ksReadyQueuesL1Bitmap: [word_t; 1];
    fn rust_clzl(x: u64) -> i64;
    static mut ksReadyQueues: [tcb_queue_t; 256];
    static mut ksReadyQueuesL2Bitmap: [[word_t; L2_BITMAP_SIZE]; 1];
    static mut ksIdleThread: *mut tcb_t;
    fn Arch_switchToThread(tcb: *mut tcb_t);
    fn Arch_switchToIdleThread();
    fn deriveCap(slot: *mut cte, cap: cap) -> DeriveCapRet;
}
pub const L2_BITMAP_SIZE: usize = (256 + (1 << 6) - 1) / (1 << 6);

pub extern "C" fn configureIdleThread(tcb: *mut tcb_t) {
    unsafe {
        let mut tcb_1 = *tcb;
        Arch_configureIdleThread(tcb);
        setThreadState(tcb, ThreadState_IdleThreadState as u64);
    }
}

pub extern "C" fn activateThread() {
    // if let Some(tcb) = unsafe { (ksCurThread) }.tcbYieldTo {
    //     schedContext_completeYieldTo(unsafe { (ksCurThread) });
    //     assert_eq!(thread_state_get_tsType(unsafe { (ksCurThread) }.tcbState), ThreadState_Running);
    // }
    unsafe {
        match thread_state_get_tsType(&((*ksCurThread).tcbState) as *const thread_state_t) {
            /*ThreadState_Running as u64 */
            1 => {
                // 线程状态为 ThreadState_Running 时无需处理
            }
            #[cfg(feature = "config_vtx")]
            ThreadState_RunningVM => {
                // 线程状态为 ThreadState_RunningVM 时无需处理
            }
            ThreadState_Restart => {
                let pc = getRestartPC(unsafe { (ksCurThread) });
                setNextPC(unsafe { (ksCurThread) }, pc);
                setThreadState((ksCurThread), ThreadState_Running as u64);
            }
            /*ThreadState_IdleThreadState */
            7 => {
                Arch_activateIdleThread(unsafe { (ksCurThread) });
            }
            _ => {
                panic!("Current thread is blocked");
            }
        }
    }
}

pub unsafe extern "C" fn suspend(target: *mut tcb_t) {
    cancelIPC(target);
    setThreadState(target, _thread_state::ThreadState_Inactive as u64);
    tcbSchedDequeue(target);
}

#[repr(C)]
pub enum _bool {
    r#false = 0,
    r#true = 1,
}
type bool_t = word_t;

pub unsafe extern "C" fn isBlocked(thread: *const tcb_t) -> bool_t {
    let tcbState = &(*thread).tcbState;
    let tsType = thread_state_get_tsType(tcbState);
    if tsType == (_thread_state::ThreadState_Inactive as u64)
        || tsType == (_thread_state::ThreadState_BlockedOnReceive as u64)
        || tsType == (_thread_state::ThreadState_BlockedOnSend as u64)
        || tsType == (_thread_state::ThreadState_BlockedOnReply as u64)
    {
        _bool::r#true as u64
    } else {
        _bool::r#false as u64
    }
}

use crate::kernel::tcb::*;
pub fn restart(target: *mut tcb_t) {
    unsafe {
        if isBlocked(target) != 0 {
            cancelIPC(target);
            #[cfg(feature = "config_kernel_mcs")]
            {
                setThreadState(target, ThreadState_Restart);
                if sc_sporadic(target.tcbSchedContext)
                    && target.tcbSchedContext != unsafe { (ksCurSC) }
                {
                    refill_unblock_check(target.tcbSchedContext);
                }
                schedContext_resume(target.tcbSchedContext);
                if isSchedulable(target) {
                    possibleSwitchTo(target);
                }
            }
            #[cfg(not(feature = "config_kernel_mcs"))]
            {
                setupReplyMaster(target);
                setThreadState(target, _thread_state::ThreadState_Restart as u64);
                tcbSchedEnqueue(target);
                unsafe {
                    possibleSwitchTo(target);
                }
            }
        }
    }
}

use crate::kernel::fastpath::seL4_Fault_get_seL4_FaultType;
use crate::kernel::fastpath::seL4_Fault_tag::seL4_Fault_NullFault;
use core::ffi::c_void;
use core::intrinsics::likely;
fn doIPCTransfer(
    sender: *mut tcb_t,
    endpoint: *mut endpoint_t,
    badge: word_t,
    grant: bool_t,
    receiver: *mut tcb_t,
) {
    let mut receiveBuffer: *mut word_t= 0 as *mut word_t;
    let mut sendBuffer: *mut word_t=0 as *mut word_t;
    unsafe {
        *receiveBuffer = lookupIPCBuffer(1, receiver);
    }
    unsafe {
        if likely(seL4_Fault_get_seL4_FaultType((*sender).tcbFault) == seL4_Fault_NullFault as u64)
        {
            *sendBuffer = lookupIPCBuffer(0, sender);
            doNormalTransfer(
                sender,
                sendBuffer,
                endpoint,
                badge,
                grant,
                receiver,
                receiveBuffer,
            );
        } else {
            doFaultTransfer(badge, sender, receiver, receiveBuffer);
        }
    }
}

#[derive(Clone,Copy)]
#[repr(C)]
pub struct endpoint {
    pub words: [u64; 2],
}
pub type endpoint_t = endpoint;

#[inline]
pub fn seL4_Fault_NullFault_new() -> seL4_Fault_t {
    seL4_Fault_t { words: [0, 0] }
}

pub unsafe extern "C" fn doReplyTransfer(
    sender: *mut tcb_t,
    receiver: *mut tcb_t,
    slot: *mut cte_t,
) {
    if seL4_Fault_get_seL4_FaultType((*receiver).tcbFault) == seL4_Fault_NullFault as u64 {
        doIPCTransfer(
            sender,
            0 as *mut endpoint_t,
            0,
            _bool::r#true as u64,
            receiver,
        );
        cteDeleteOne(slot);
        setThreadState(receiver, _thread_state::ThreadState_Running as u64);
        possibleSwitchTo(receiver);
    } else {
        cteDeleteOne(slot);
        let restart: bool_t = handleFaultReply(receiver, sender);
        (*receiver).tcbFault = seL4_Fault_NullFault_new();
        if restart != 0 {
            setThreadState(receiver, _thread_state::ThreadState_Restart as u64);
            possibleSwitchTo(receiver);
        } else {
            setThreadState(receiver, _thread_state::ThreadState_Inactive as u64);
        }
    }
}

use crate::failures::exception_t;
use crate::kernel::fastpath::seL4_MessageInfo_t;
pub const seL4_MsgMaxLength: u64 = 120;

#[inline]
pub fn seL4_MessageInfo_set_length(
    mut seL4_MessageInfo: seL4_MessageInfo_t,
    v64: u64,
) -> seL4_MessageInfo_t {
    seL4_MessageInfo.words[0] &= !0x7fu64;
    seL4_MessageInfo.words[0] |= v64 & 0x7fu64;
    seL4_MessageInfo
}

#[inline]
pub fn messageInfoFromWord(w: word_t) -> seL4_MessageInfo_t {
    let mut mi: seL4_MessageInfo_t = seL4_MessageInfo_t { words: [w] };
    unsafe {
        let len: word_t = seL4_MessageInfo_get_length(mi);
        if len > seL4_MsgMaxLength {
            mi = seL4_MessageInfo_set_length(mi, seL4_MsgMaxLength);
        }
    }
    mi
}

pub type register_t = u32;
pub const msgInfoRegister: u32 = 1;
pub const badgeRegister: u32 = 0;
const EXCEPTION_NONE: u64 = 0;
use crate::kernel::fastpath::wordFromMessageInfo;
use crate::kernel::tcb::copyMRs;
use core::intrinsics::unlikely;
use core::ptr::null_mut;
//type exception_t = word_t;
pub unsafe fn getRegister(thread: *mut tcb_t, reg: register_t) -> word_t {
    (*thread).tcbArch.tcbContext.registers[reg as usize]
}
pub unsafe fn setRegister(thread: *mut tcb_t, reg: register_t, w: word_t) {
    (*thread).tcbArch.tcbContext.registers[reg as usize] = w;
}
pub fn doNormalTransfer(
    sender: *mut tcb_t,
    sendBuffer: *mut word_t,
    endpoint: *mut endpoint_t,
    badge: word_t,
    canGrant: bool_t,
    receiver: *mut tcb_t,
    receiveBuffer: *mut word_t,
) {
    unsafe {
        let mut msgTransferred: word_t;
        let mut tag: seL4_MessageInfo_t;
        let mut status: exception_t;
        tag = messageInfoFromWord(getRegister(sender, msgInfoRegister));

        if canGrant != 0 {
            status = lookupExtraCaps(sender, sendBuffer, tag);
            if unlikely(status != EXCEPTION_NONE) {
                current_extra_caps.excaprefs[0] = null_mut();
            }
        } else {
            current_extra_caps.excaprefs[0] = null_mut();
        }

        msgTransferred = copyMRs(
            sender,
            sendBuffer,
            receiver,
            receiveBuffer,
            seL4_MessageInfo_get_length(tag),
        );

        tag = transferCaps(tag, endpoint, receiver, receiveBuffer);

        tag = seL4_MessageInfo_set_length(tag, msgTransferred);
        setRegister(receiver, msgInfoRegister, wordFromMessageInfo(tag));
        setRegister(receiver, badgeRegister, badge);
    }
}

#[inline]
pub fn seL4_MessageInfo_new(
    label: u64,
    capsUnwrapped: u64,
    extraCaps: u64,
    length: u64,
) -> seL4_MessageInfo_t {
    let ret: u64 = 0
        | (label & 0xfffffffffffffu64) << 12
        | (capsUnwrapped & 0x7u64) << 9
        | (extraCaps & 0x3u64) << 7
        | (length & 0x7fu64) << 0;
    seL4_MessageInfo_t { words: [ret] }
}

pub fn doFaultTransfer(
    badge: word_t,
    sender: *mut tcb_t,
    receiver: *mut tcb_t,
    receiverIPCBuffer: *mut word_t,
) {
    unsafe {
        let mut sent: word_t;
        let mut msgInfo: seL4_MessageInfo_t;

        sent = setMRs_fault(sender, receiver, receiverIPCBuffer);
        msgInfo = seL4_MessageInfo_new(
            seL4_Fault_get_seL4_FaultType((*sender).tcbFault),
            0,
            0,
            sent,
        );
        setRegister(receiver, msgInfoRegister, wordFromMessageInfo(msgInfo));
        setRegister(receiver, badgeRegister, badge);
    }
}

#[inline]
pub fn seL4_MessageInfo_set_extraCaps(
    mut seL4_MessageInfo: seL4_MessageInfo_t,
    v64: u64,
) -> seL4_MessageInfo_t {
    seL4_MessageInfo.words[0] &= !0x180u64;
    seL4_MessageInfo.words[0] |= (v64 << 7) & 0x180u64;
    seL4_MessageInfo
}
#[inline]
pub fn seL4_MessageInfo_set_capsUnwrapped(
    mut seL4_MessageInfo: seL4_MessageInfo_t,
    v64: u64,
) -> seL4_MessageInfo_t {
    seL4_MessageInfo.words[0] &= !0xe00u64;
    seL4_MessageInfo.words[0] |= (v64 << 9) & 0xe00u64;
    seL4_MessageInfo
}
const seL4_MsgExtraCapBits: usize = 2;
pub const seL4_MsgMaxExtraCaps: usize = (1usize << seL4_MsgExtraCapBits) - 1;
use crate::kernel::fastpath::cap_endpoint_cap_get_capEPBadge;
use crate::kernel::fastpath::cap_endpoint_cap_get_capEPPtr;
use crate::kernel::fastpath::cap_tag::cap_endpoint_cap;
#[inline]
pub fn seL4_MessageInfo_get_capsUnwrapped(seL4_MessageInfo: seL4_MessageInfo_t) -> u64 {
    (seL4_MessageInfo.words[0] & 0xe00u64) >> 9
}
use crate::kernel::tcb::cteInsert;
pub fn transferCaps(
    info: seL4_MessageInfo_t,
    endpoint: *mut endpoint_t,
    receiver: *mut tcb_t,
    receiveBuffer: *mut word_t,
) -> seL4_MessageInfo_t {
    unsafe {
        let mut i: word_t=0;
        let mut destSlot: *mut cte_t;

        let mut info = seL4_MessageInfo_set_extraCaps(info, 0);
        info = seL4_MessageInfo_set_capsUnwrapped(info, 0);

        if likely(!current_extra_caps.excaprefs[0].is_null() && receiveBuffer.is_null()) {
            return info;
        }

        destSlot = getReceiveSlots(receiver, receiveBuffer);

        for i in 0..seL4_MsgMaxExtraCaps {
            let slot = current_extra_caps.excaprefs[i];
            if slot.is_null() {
                break;
            }

            let cap = (*slot).cap;

            if cap_get_capType(cap) == cap_endpoint_cap as u64
                && cap_endpoint_cap_get_capEPPtr(cap) == endpoint as u64
            {
                // 如果这是发送消息的端点的能力，则仅传输标记（badge），而不是能力
                setExtraBadge(
                    receiveBuffer,
                    cap_endpoint_cap_get_capEPBadge(cap),
                    i as u64,
                );
                //再tcb中有定义，但是用不了
                info = seL4_MessageInfo_set_capsUnwrapped(
                    info,
                    seL4_MessageInfo_get_capsUnwrapped(info) | (1 << i),
                );
            } else {
                let dc_ret = deriveCap(slot, cap);

                if destSlot.is_null() {
                    break;
                }

                if dc_ret.status != EXCEPTION_NONE {
                    break;
                }

                if cap_get_capType(dc_ret.cap) == cap_tag_t::cap_null_cap as u64 {
                    break;
                }

                cteInsert(dc_ret.cap, slot, destSlot);

                destSlot = null_mut();
            }
        }

        seL4_MessageInfo_set_extraCaps(info, i)
    }
}

pub fn doNBRecvFailedTransfer(thread: &mut tcb_t) {
    // 将标记寄存器设置为0，表示没有消息
    unsafe {
        setRegister(thread, badgeRegister, 0);
    }
}
pub fn ready_queues_index(dom: word_t, prio: word_t) -> word_t {
    prio
}
pub fn l1index_to_prio(l1index: word_t) -> word_t {
    l1index << 6
}
pub fn invert_l1index(l1index: word_t) -> word_t {
    L2_BITMAP_SIZE as u64 - 1 - l1index
}
use crate::types::wordBits;

use super::notification::cancelIPC;
pub fn getHighestPrio(dom: word_t) -> prio_t {
    unsafe {
        let l1index: word_t =
            (wordBits as i64 - 1 - rust_clzl((ksReadyQueuesL1Bitmap)[dom as usize])) as u64;
        let l1index_inverted: word_t = invert_l1index(l1index);
        let l2index: word_t = (wordBits as i64
            - 1
            - rust_clzl((ksReadyQueuesL2Bitmap)[dom as usize][l1index_inverted as usize]))
            as u64;
        l1index_to_prio(l1index) | l2index
    }
}
pub fn chooseThread() {
    unsafe {
        let dom: word_t = 0;
        if (ksReadyQueuesL1Bitmap)[dom as usize] != 0 {
            let prio: word_t = getHighestPrio(dom);
            let thread: *mut tcb_t = (ksReadyQueues)[ready_queues_index(dom, prio) as usize].head;
            switchToThread(thread);
        } else {
            switchToIdleThread();
        }
    }
}

pub fn nextDomain() {
    unsafe {
        ksDomScheduleIdx += 1;
        if ksDomScheduleIdx >= ksDomScheduleLength {
            ksDomScheduleIdx = 0;
        }

        ksWorkUnitsCompleted = 0;
        ksCurDomain = ksDomSchedule[ksDomScheduleIdx as usize].domain;

        #[cfg(not(CONFIG_KERNEL_MCS))]
        {
            ksDomainTime = ksDomSchedule[ksDomScheduleIdx as usize].length;
        }
    }
}
pub fn scheduleChooseNewThread() {
    unsafe {
        if ksDomainTime == 0 {
            nextDomain();
        }
        chooseThread();
    }
}

const SchedulerAction_ResumeCurrentThread: *mut tcb_t = 0 as *mut tcb_t;
const SchedulerAction_ChooseNewThread: *mut tcb_t = 1 as *mut tcb_t;
fn isHighestPrio(dom: word_t, prio: prio_t) -> bool_t {
    unsafe { ((ksReadyQueuesL1Bitmap)[dom as usize] == 0 || prio >= getHighestPrio(dom)) as u64 }
}
pub fn schedule() {
    unsafe {
        #[cfg(CONFIG_KERNEL_MCS)]
        unsafe {
            awaken();
            checkDomainTime();
        }

        if (ksSchedulerAction) != SchedulerAction_ResumeCurrentThread {
            let was_runnable: bool_t=0;
            // if isSchedulable((ksCurThread)) {
            //     was_runnable = true;
            //     SCHED_ENQUEUE_CURRENT_TCB;
            // } else {
            //     was_runnable = false;
            // }

            if (ksSchedulerAction) == SchedulerAction_ChooseNewThread {
                #[cfg(CONFIG_KERNEL_MCS)]
                unsafe {
                    scheduleChooseNewThread()
                };
            } else {
                let candidate = (ksSchedulerAction);
                // assert(isSchedulable(candidate));

                let fastfail = (ksCurThread) == (ksIdleThread)
                    || (*candidate).tcbPriority < (*ksCurThread).tcbPriority;

                if fastfail && isHighestPrio(ksCurDomain, (*candidate).tcbPriority) == 0 {
                    tcbSchedEnqueue(candidate); //tcb里有定义
                    (ksSchedulerAction) = SchedulerAction_ChooseNewThread;
                    #[cfg(CONFIG_KERNEL_MCS)]
                    unsafe {
                        scheduleChooseNewThread()
                    };
                } else if was_runnable != 0
                    && (*candidate).tcbPriority == (*ksCurThread).tcbPriority
                {
                    tcbSchedAppend(candidate); //tcb里有定义
                    (ksSchedulerAction) = SchedulerAction_ChooseNewThread;
                    #[cfg(CONFIG_KERNEL_MCS)]
                    unsafe {
                        scheduleChooseNewThread()
                    };
                } else {
                    // assert(candidate != (ksCurThread));
                    switchToThread(candidate);
                }
            }
        }
        (ksSchedulerAction) = SchedulerAction_ResumeCurrentThread;
    }
}

pub fn switchToThread(thread: *mut tcb_t) {
    unsafe {
        Arch_switchToThread(thread);
    }
    tcbSchedDequeue(thread); //tcb有定义
    unsafe {
        (ksCurThread) = thread;
    }
}

pub fn switchToIdleThread() {
    unsafe {
        Arch_switchToIdleThread();
        (ksCurThread) = (ksIdleThread);
    }
}

fn isSchedulable(thread: *const tcb_t) -> bool_t {
    let state = unsafe { thread_state_get_tsType(&(*thread).tcbState) };
    if state == _thread_state::ThreadState_Running as u64
        || state == _thread_state::ThreadState_Restart as u64
    {
        true as bool_t
    } else {
        false as bool_t
    }
}

pub fn setDomain(tptr: *mut tcb_t, dom: *mut dom_t) {
    unsafe {
        tcbSchedDequeue(tptr); //tcb
        (*tptr).tcbDomain = *dom;
        if isSchedulable(tptr) != 0 {
            tcbSchedEnqueue(tptr); //tcb
        }
        if tptr == unsafe { (ksCurThread) } {
            unsafe {
                rescheduleRequired();
            }
        }
    }
}

pub fn setMCPriority(tptr: *mut tcb_t, mcp: prio_t) {
    unsafe {
        (*tptr).tcbMCP = mcp;
    }
}

pub fn setPriority(tptr: *mut tcb_t, prio: *mut prio_t) {
    unsafe {
        tcbSchedDequeue(tptr);
        (*tptr).tcbPriority = *prio;
        if isSchedulable(tptr) != 0 {
            if tptr == unsafe { (ksCurThread) } {
                unsafe {
                    rescheduleRequired();
                }
            } else {
                unsafe {
                    possibleSwitchTo(tptr);
                }
            }
        }
    }
}
pub unsafe extern "C" fn possibleSwitchTo(target: *mut tcb_t) {
    //ignore smp
    if ksCurDomain != (*target).tcbDomain {
        tcbSchedEnqueue(target);
    } else if (ksSchedulerAction) != SchedulerAction_ResumeCurrentThread {
        rescheduleRequired();
        tcbSchedEnqueue(target);
    } else {
        (ksSchedulerAction) = target;
    }
}
#[inline]
pub fn thread_state_ptr_set_tsType(thread_state_ptr: *mut thread_state_t, v64: u64) {
    unsafe {
        (*thread_state_ptr).words[0] &= !0xfu64;
        (*thread_state_ptr).words[0] |= v64 & 0xfu64;
    }
}

pub fn setThreadState(tptr: *mut tcb_t, ts: _thread_state_t) {
    unsafe {
        thread_state_ptr_set_tsType(&mut ((*tptr).tcbState) as *mut thread_state, ts);
        scheduleTCB(tptr);
    }
}
pub fn scheduleTCB(tptr: *mut tcb_t) {
    unsafe {
        if tptr == ksCurThread
            && unsafe { (ksSchedulerAction) } == SchedulerAction_ResumeCurrentThread
            && isSchedulable(tptr) == 0
        {
            unsafe {
                rescheduleRequired();
            }
        }
    }
}

const CONFIG_TIME_SLICE: u64 = 5;
pub fn timerTick() {
    unsafe {
        if thread_state_get_tsType(&(*(ksCurThread)).tcbState)
            == _thread_state::ThreadState_Running as u64
        {
            if (*(ksCurThread)).tcbTimeSlice > 1 {
                (*(ksCurThread)).tcbTimeSlice -= 1;
            } else {
                (*(ksCurThread)).tcbTimeSlice = CONFIG_TIME_SLICE;
                tcbSchedAppend((ksCurThread));
                rescheduleRequired();
            }
        }
    }
}

pub fn rescheduleRequired() {
    unsafe {
        if (ksSchedulerAction) != SchedulerAction_ResumeCurrentThread
            && (ksSchedulerAction) != SchedulerAction_ChooseNewThread
        {
            tcbSchedEnqueue((ksSchedulerAction));
        }
        (ksSchedulerAction) = SchedulerAction_ChooseNewThread;
    }
}

#[inline(always)]
pub fn prio_to_l1index(prio:u64)->u64
{
    prio >> wordRadix
}