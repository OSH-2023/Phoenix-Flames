

//thread.c

pub type prio_t = word_t;
#[repr(C)]
pub struct tcb_t {
    pub tcbArch: arch_tcb_t,
    pub tcbState: thread_state_t,
    pub tcbBoundNotification: *mut notification_t,
    pub tcbFault: seL4_Fault_t,
    pub tcbLookupFailure: lookup_fault_t,
    pub tcbDomain: types::dom_t,
    pub tcbMCP: types::prio_t,
    pub tcbPriority: types::prio_t,
    pub tcbTimeSlice: word_t,
    pub tcbFaultHandler: types::cptr_t,
    pub tcbIPCBuffer: word_t,

    pub tcbSchedNext: *mut tcb_t,
    pub tcbSchedPrev: *mut tcb_t,
    pub tcbEPNext: *mut tcb_t,
    pub tcbEPPrev: *mut tcb_t,

    pub tcbDebugNext: *mut tcb_t,
    pub tcbDebugPrev: *mut tcb_t,
    pub tcbName: *mut u8, //C语言中是char tcbName[]，这里直接翻译成指针了
}

pub fn configureIdleThread(tcb: *mut tcb_t) {
    unsafe {
        Arch_configureIdleThread(tcb);
        setThreadState(tcb, ThreadState_IdleThreadState);
    }
}

pub fn activateThread() {
    if let Some(tcb) = unsafe { NODE_STATE(ksCurThread) }.tcbYieldTo {
        schedContext_completeYieldTo(unsafe { NODE_STATE(ksCurThread) });
        assert_eq!(thread_state_get_tsType(unsafe { NODE_STATE(ksCurThread) }.tcbState), ThreadState_Running);
    }

    match thread_state_get_tsType(unsafe { NODE_STATE(ksCurThread) }.tcbState) {
        ThreadState_Running => {
            // 线程状态为 ThreadState_Running 时无需处理
        }
        #[cfg(feature = "config_vtx")]
        ThreadState_RunningVM => {
            // 线程状态为 ThreadState_RunningVM 时无需处理
        }
        ThreadState_Restart => {
            let pc = getRestartPC(unsafe { NODE_STATE(ksCurThread) });
            setNextPC(unsafe { NODE_STATE(ksCurThread) }, pc);
            setThreadState(unsafe { NODE_STATE(ksCurThread) }, ThreadState_Running);
        }
        ThreadState_IdleThreadState => {
            Arch_activateIdleThread(unsafe { NODE_STATE(ksCurThread) });
        }
        _ => {
            fail("Current thread is blocked");
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn suspend(target: *mut tcb_t) {
    cancelIPC(target);
    setThreadState(target, _thread_state::ThreadState_Inactive as u64);
    tcbSchedDequeue(target);
}

pub fn restart(target: &mut tcb_t) {
    if isStopped(target) {
        cancelIPC(target);
        #[cfg(feature = "config_kernel_mcs")]
        {
            setThreadState(target, ThreadState_Restart);
            if sc_sporadic(target.tcbSchedContext)
                && target.tcbSchedContext != unsafe { NODE_STATE(ksCurSC) }
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
            setThreadState(target, ThreadState_Restart);
            SCHED_ENQUEUE(target);
            unsafe{possibleSwitchTo(target);}
        }
    }
}
fn doIPCTransfer(
    sender: &mut tcb_t,
    endpoint: &mut endpoint_t,
    badge: word_t,
    grant: bool_t,
    receiver: &mut tcb_t,
) {
    let mut receiveBuffer: *mut c_void;
    let mut sendBuffer: *mut c_void;

    receiveBuffer = lookupIPCBuffer(true, receiver);

    if likely(seL4_Fault_get_seL4_FaultType(sender.tcbFault) == seL4_Fault_NullFault) {
        sendBuffer = lookupIPCBuffer(false, sender);
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

#[repr(C)]
pub struct endpoint {
    words: [u64; 2],
}
pub type endpoint_t = endpoint;

#[no_mangle]
pub unsafe extern "C" fn doReplyTransfer(
    sender: *mut tcb_t,
    receiver: *mut tcb_t,
    slot: *mut cte_t,
) {
    if seL4_Fault_get_seL4_FaultType(&(*receiver).tcbFault) == seL4_Fault_NullFault {
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
pub fn doNormalTransfer(
    sender: &mut tcb_t,
    sendBuffer: *mut word_t,
    endpoint: &mut endpoint_t,
    badge: word_t,
    canGrant: bool_t,
    receiver: &mut tcb_t,
    receiveBuffer: *mut word_t,
) {
    let mut msgTransferred: word_t;
    let mut tag: seL4_MessageInfo_t;
    let mut status: exception_t;

    tag = messageInfoFromWord(getRegister(sender, msgInfoRegister));

    if canGrant {
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

pub fn doFaultTransfer(badge: word_t, sender: &mut tcb_t, receiver: &mut tcb_t, receiverIPCBuffer: *mut word_t) {
    let mut sent: word_t;
    let mut msgInfo: seL4_MessageInfo_t;

    sent = setMRs_fault(sender, receiver, receiverIPCBuffer);
    msgInfo = seL4_MessageInfo_new(
        seL4_Fault_get_seL4_FaultType(sender.tcbFault),
        0,
        0,
        sent,
    );
    setRegister(receiver, msgInfoRegister, wordFromMessageInfo(msgInfo));
    setRegister(receiver, badgeRegister, badge);
}

pub fn doFaultTransfer(badge: word_t, sender: &mut tcb_t, receiver: &mut tcb_t, receiverIPCBuffer: *mut word_t) {
    let mut sent: word_t;
    let mut msgInfo: seL4_MessageInfo_t;

    sent = setMRs_fault(sender, receiver, receiverIPCBuffer);
    msgInfo = seL4_MessageInfo_new(
        seL4_Fault_get_seL4_FaultType(sender.tcbFault),
        0,
        0,
        sent,
    );
    setRegister(receiver, msgInfoRegister, wordFromMessageInfo(msgInfo));
    setRegister(receiver, badgeRegister, badge);
}
pub fn transferCaps(
    info: seL4_MessageInfo_t,
    endpoint: &mut endpoint_t,
    receiver: &mut tcb_t,
    receiveBuffer: *mut word_t,
) -> seL4_MessageInfo_t {
    let mut i: word_t;
    let mut destSlot: *mut cte_t;

    info = seL4_MessageInfo_set_extraCaps(info, 0);
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

        let cap = slot.cap;

        if cap_get_capType(cap) == cap_endpoint_cap && EP_PTR(cap_endpoint_cap_get_capEPPtr(cap)) == endpoint {
            // 如果这是发送消息的端点的能力，则仅传输标记（badge），而不是能力
            setExtraBadge(receiveBuffer, cap_endpoint_cap_get_capEPBadge(cap), i);
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

            if cap_get_capType(dc_ret.cap) == cap_null_cap {
                break;
            }

            cteInsert(dc_ret.cap, slot, destSlot);

            destSlot = null_mut();
        }
    }

    seL4_MessageInfo_set_extraCaps(info, i)
}

pub fn doNBRecvFailedTransfer(thread: &mut tcb_t) {
    // 将标记寄存器设置为0，表示没有消息
    setRegister(thread, badgeRegister, 0);
}

pub fn nextDomain() {
    ksDomScheduleIdx += 1;
    if ksDomScheduleIdx >= ksDomScheduleLength {
        ksDomScheduleIdx = 0;
    }

    ksWorkUnitsCompleted = 0;
    ksCurDomain = ksDomSchedule[ksDomScheduleIdx].domain;

    #[cfg(not(CONFIG_KERNEL_MCS))]
    {
        ksDomainTime = ksDomSchedule[ksDomScheduleIdx].length;
    }
}
pub fn scheduleChooseNewThread() {
    if ksDomainTime == 0 {
        nextDomain();
    }
    chooseThread();
}

const SchedulerAction_ResumeCurrentThread: *mut tcb_t = 0 as *mut tcb_t;
const SchedulerAction_ChooseNewThread: *mut tcb_t = 1 as *mut tcb_t;
pub fn schedule() {
    #[cfg(CONFIG_KERNEL_MCS)]
    unsafe {
        awaken();
        checkDomainTime();
    }

    if NODE_STATE(ksSchedulerAction) != SchedulerAction_ResumeCurrentThread {
        let was_runnable: bool_t;
        if isSchedulable(NODE_STATE(ksCurThread)) {
            was_runnable = true;
            SCHED_ENQUEUE_CURRENT_TCB;
        } else {
            was_runnable = false;
        }

        if NODE_STATE(ksSchedulerAction) == SchedulerAction_ChooseNewThread {
            #[cfg(CONFIG_KERNEL_MCS)]
            unsafe { scheduleChooseNewThread() };
        } else {
            let candidate = NODE_STATE(ksSchedulerAction);
            assert(isSchedulable(candidate));

            let fastfail = NODE_STATE(ksCurThread) == NODE_STATE(ksIdleThread)
                || candidate.tcbPriority < NODE_STATE(ksCurThread).tcbPriority;

            if fastfail && !isHighestPrio(ksCurDomain, candidate.tcbPriority) {
                SCHED_ENQUEUE(candidate);
                NODE_STATE(ksSchedulerAction) = SchedulerAction_ChooseNewThread;
                #[cfg(CONFIG_KERNEL_MCS)]
                unsafe { scheduleChooseNewThread() };
            } else if was_runnable && candidate.tcbPriority == NODE_STATE(ksCurThread).tcbPriority {
                SCHED_APPEND(candidate);
                NODE_STATE(ksSchedulerAction) = SchedulerAction_ChooseNewThread;
                #[cfg(CONFIG_KERNEL_MCS)]
                unsafe { scheduleChooseNewThread() };
            } else {
                assert(candidate != NODE_STATE(ksCurThread));
                switchToThread(candidate);
            }
        }
    }
    NODE_STATE(ksSchedulerAction) = SchedulerAction_ResumeCurrentThread;
}

pub fn switchToThread(thread: tcb_t) {
    unsafe {
        Arch_switchToThread(thread);
    }
    tcbSchedDequeue(thread);
    unsafe {
        NODE_STATE(ksCurThread) = thread;
    }
}

pub fn switchToIdleThread() {
    unsafe {
        Arch_switchToIdleThread();
        NODE_STATE(ksCurThread) = NODE_STATE(ksIdleThread);
    }
}

pub fn setDomain(tptr: tcb_t, dom: dom_t) {
    tcbSchedDequeue(tptr);
    tptr.tcbDomain = dom;
    if isSchedulable(tptr) {
        SCHED_ENQUEUE(tptr);
    }
    if tptr == unsafe { NODE_STATE(ksCurThread) } {
        unsafe {rescheduleRequired();}
    }
}


pub fn setMCPriority(tptr: *mut tcb_t, mcp: prio_t) {
    unsafe{(*tptr).tcbMCP = mcp;}
}

pub fn setPriority(tptr: tcb_t, prio: prio_t) {
    tcbSchedDequeue(tptr);
    tptr.tcbPriority = prio;
    if isRunnable(tptr) {
        if tptr == unsafe { NODE_STATE(ksCurThread) } {
            unsafe {rescheduleRequired();}
        } else {
           unsafe{ possibleSwitchTo(tptr);}
        }
    }
}
pub unsafe extern "C" fn possibleSwitchTo(target: *mut tcb_t) {
    //ignore smp
    if ksCurDomain != (*target).tcbDomain {
        tcbSchedEnqueue(target);
    } else if node_state!(ksSchedulerAction) != SchedulerAction_ResumeCurrentThread {
        rescheduleRequired();
        tcbSchedEnqueue(target);
    } else {
        node_state!(ksSchedulerAction) = target;
    }
}
pub fn setThreadState(tptr: tcb_t, ts: _thread_state_t) {
    thread_state_ptr_set_tsType(&tptr.tcbState, ts);
    scheduleTCB(tptr);
}
pub fn scheduleTCB(tptr: tcb_t) {
    if tptr == unsafe { NODE_STATE(ksCurThread) } &&
        unsafe { NODE_STATE(ksSchedulerAction) } == SchedulerAction_ResumeCurrentThread &&
        !isSchedulable(tptr)
    {
        unsafe {rescheduleRequired();}
    }
}
const CONFIG_TIME_SLICE: u64 = 5;
#[no_mangle]
pub unsafe extern "C" fn timerTick() {
    if thread_state_get_tsType(&(*node_state!(ksCurThread)).tcbState)
        == _thread_state::ThreadState_Running as u64
    {
        if (*node_state!(ksCurThread)).tcbTimeSlice > 1 {
            (*node_state!(ksCurThread)).tcbTimeSlice -= 1;
        } else {
            (*node_state!(ksCurThread)).tcbTimeSlice = CONFIG_TIME_SLICE;
            tcbSchedAppend(node_state!(ksCurThread));
            rescheduleRequired();
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rescheduleRequired() {
    if node_state!(ksSchedulerAction) != SchedulerAction_ResumeCurrentThread
        && node_state!(ksSchedulerAction) != SchedulerAction_ChooseNewThread
    {
        tcbSchedEnqueue(node_state!(ksSchedulerAction));
    }
    node_state!(ksSchedulerAction) = SchedulerAction_ChooseNewThread;
}