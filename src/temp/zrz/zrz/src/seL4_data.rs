#[allow(non_snake_case)]
#[allow(non_camel_case_types)]

pub mod data_type{ 
    //notification
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct notification {
        pub words: [u64;4],
    }

    //thread_state
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct thread_state {
        pub words: [u64;3],
    }

    //seL4_Fault
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct seL4_Fault {
        pub words: [u64;2],
    }

    //lookup_fault
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct lookup_fault {
        pub words: [u64;2],
    }
    pub type lookup_fault_t = lookup_fault;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct user_fpu_state {
        pub state: [u8;512],
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct user_context {
        pub fpuState: user_fpu_state,
        pub registers: [u64;24],
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct arch_tcb_t {
        pub tcbContext: user_context,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct tcb{
        pub tcbArch: arch_tcb_t,
        pub tcbState: thread_state,
        pub tcbBoundNotification: *mut notification,
        pub tcbFault: seL4_Fault,
        pub tcbLookupFailure:lookup_fault,
        pub tcbDomain: u32,
        pub tcbMCP: u32,
        pub tcbPriority: u32,
        pub tcbTimeSlice: u32,
        pub tcbFaultHandler: u32,
        pub tcbIPCBuffer: u32,
        pub tcbSchedNext: *mut tcb,
        pub tcbSchedPrev: *mut tcb,
        pub tcbEPNext: *mut tcb,
        pub tcbEPPrev: *mut tcb,
    }

    pub type tcb_t = tcb;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct cap{
        pub words: [u64;2],
    }

    pub type cap_t = cap;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct mdb_node{
        pub words: [u64;2],
    }

    pub type mdb_node_t = mdb_node;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct cte {
        pub cap: cap_t,
        pub cteMDBNode: mdb_node_t,
    }

    pub type cte_t = cte;

    pub type word_t = u64;
    pub type vptr_t = word_t;
    pub type paddr_t = word_t;
    pub type pptr_t = word_t;
    pub type cptr_t = word_t;
    pub type dev_id_t = word_t;
    pub type cpu_id_t = word_t;
    pub type logical_id_t = u32;
    pub type node_id_t = word_t;
    pub type dom_t = word_t;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct resolveAddressBits_ret{
        pub status: exception_t,
        pub slot: *mut cte_t,
        pub bitsRemaining: word_t,
    }
    pub type resolveAddressBits_ret_t = resolveAddressBits_ret;

    #[repr(C)]
    #[derive(Copy, Clone)]
    #[repr(u64)]
    pub enum cap_tag {
        cap_null_cap(u64)= 0,
        cap_untyped_cap(u64) = 2,
        cap_endpoint_cap(u64) = 4,
        cap_notification_cap(u64) = 6,
        cap_reply_cap(u64) = 8,
        cap_cnode_cap(u64) = 10,
        cap_thread_cap(u64) = 12,
        cap_irq_control_cap(u64) = 14,
        cap_irq_handler_cap(u64) = 16,
        cap_zombie_cap(u64) = 18,
        cap_domain_cap(u64) = 20,
        cap_frame_cap(u64) = 1,
        cap_page_table_cap(u64) = 3,
        cap_asid_control_cap(u64) = 11,
        cap_asid_pool_cap(u64) = 13
    }
    pub type cap_tag_t = word_t;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub enum exception {
        EXCEPTION_NONE,
        EXCEPTION_FAULT,
        EXCEPTION_LOOKUP_FAULT,
        EXCEPTION_SYSCALL_ERROR,
        EXCEPTION_PREEMPTED
    }
    pub type exception_t = word_t;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub enum lookup_fault_tag {
        lookup_fault_invalid_root = 0,
        lookup_fault_missing_capability = 1,
        lookup_fault_depth_mismatch = 2,
        lookup_fault_guard_mismatch = 3
    }
    pub type lookup_fault_tag_t = word_t;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct lookupCap_ret{
        pub status: exception_t,
        pub cap: cap_t,
    }
    pub type lookupCap_ret_t = lookupCap_ret;

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct lookupSlot_raw_ret{
        pub status:exception_t,
        pub slot:*mut cte_t,
    }
    pub type lookupSlot_raw_ret_t = lookupSlot_raw_ret;

    #[repr(C)]
    #[derive(Copy,Clone)]
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
        tcbCNodeEntries
    }
    pub type tcb_cnode_index_t = word_t;

    #[repr(C)]
    #[derive(Copy,Clone)]
    pub struct lookupCapAndSlot_ret {
        pub status: exception_t,
        pub cap: cap_t,
        pub slot: *mut cte_t,
    }
    pub type lookupCapAndSlot_ret_t = lookupCapAndSlot_ret;

    #[repr(C)]
    #[derive(Copy,Clone)]
    pub struct lookupSlot_ret{
        pub status: exception_t,
        pub slot: *mut cte_t,
    }
    pub type lookupSlot_ret_t = lookupSlot_ret;

    #[repr(C)]
    #[derive(Copy,Clone)]
    pub struct syscall_error{
        pub invalidArgumentNumber:word_t,
        pub invalidCapNumber:word_t,
        pub rangeErrorMin:word_t,
        pub rangeErrorMax:word_t,
        pub memoryLeft:word_t,
        pub failedLookupWasSource:word_t,
        pub r#type:word_t,
    }
    pub type syscall_error_t = syscall_error;
}