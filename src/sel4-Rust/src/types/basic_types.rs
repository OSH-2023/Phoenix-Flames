//done

#![allow(unused)]
//turn off naming warnings
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

pub use super::stdint::*;

//from mode/types.h
pub const wordRadix: u64 = 6;

//from arch/types.h
pub type word_t = u64;
pub type sword_t = i64;
pub type vptr_t = u64;
pub type paddr_t = u64;
pub type pptr_t = u64;
pub type cptr_t = u64;
pub type dev_id_t = u64;
pub type cpu_id_t = u64;
pub type node_id_t = u64;
pub type dom_t = u64;
pub type timestamp_t = u64;

pub const wordBits: u64 = 64;

#[derive(Clone)]
pub struct kernel_frame {
    pub paddr: paddr_t,
    pub pptr: pptr_t,
    pub userAvailable: i32,
}
pub type kernel_frame_t = kernel_frame;

//from basic_types.h
pub type bool_t = word_t;
/**
 * A region [start..end) of kernel-virtual memory.
 *
 * Empty when start == end. If end < start, the region wraps around, that is,
 * it represents the addresses in the set [start..-1] union [0..end). This is
 * possible after address translation and fine for e.g. device memory regions.
 */
#[derive(Clone)]
pub struct region {
    pub start: pptr_t, /* inclusive */
    pub end: pptr_t,   /* exclusive */
}
pub type region_t = region;

/** A region [start..end) of physical memory addresses. */
#[derive(Clone)]
pub struct p_region {
    pub start: paddr_t, /* inclusive */
    pub end: paddr_t,   /* exclusive */
}
pub type p_region_t = p_region;

/** A region [start..end) of user-virtual addresses. */
#[derive(Clone)]
pub struct v_region {
    pub start: vptr_t, /* inclusive */
    pub end: vptr_t,   /* exclusive */
}
pub type v_region_t = v_region;

pub const REG_EMPTY: region_t = region { start: 0, end: 0 };
pub const P_REG_EMPTY: p_region_t = p_region { start: 0, end: 0 };

/* for libsel4 headers that the kernel shares */
pub type seL4_Uint8 = u8;
pub type seL4_Uint16 = u16;
pub type seL4_Uint32 = u32;
pub type seL4_Word = u64;
pub type seL4_CPtr = cptr_t;
pub type seL4_NodeId = node_id_t;
pub type seL4_PAddr = paddr_t;
pub type seL4_Domain = dom_t;
