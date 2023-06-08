//to be done

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
#[derive(Clone)]
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
    tcbBoundNotification: *const notification_t,

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
    tcbEPNext: *const tcb,
    tcbEPPrev: *const tcb,
}
pub type tcb_t = tcb;

// 2. from arch/object/structures.h
#[derive(Clone)]
pub struct arch_tcb {
    pub tcbContext: user_context_t,
}
pub type arch_tcb_t = arch_tcb;

// 3. from arch/object/structures_gen.h
#[derive(Clone)]
pub struct mdb_node {
    pub words: [u64; 2],
}
pub type mdb_node_t = mdb_node;

#[derive(Clone)]
pub struct notification {
    pub words:[u64;4]
}
pub type notification_t=notification;

#[derive(Clone)]
pub struct thread_state {
    pub words: [u64; 3],
}
type thread_state_t = thread_state;

#[derive(Clone)]
pub struct cap {
    pub words: [u64; 2],
}
pub type cap_t = cap;

#[derive(Clone)]
pub struct lookup_fault {
    pub words:[u64;2]
}
type lookup_fault_t=lookup_fault ;

#[derive(Clone)]
pub struct seL4_Fault {
    pub words:[u64;2]
}
pub type seL4_Fault_t=seL4_Fault;


