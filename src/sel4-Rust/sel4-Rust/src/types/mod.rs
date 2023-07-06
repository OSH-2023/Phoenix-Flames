//to be done

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

pub mod basic_types;
pub mod compound_types;
pub mod stdint;

pub use basic_types::*;
pub use compound_types::*;

/*
Including contents from:
1. api/types.h
2. arch/api/types.h
 */

// 1. from api/types.h
pub type prio_t = word_t;
pub type ticks_t = u64;
pub type time_t = u64;

// #[derive(Clone)]
// pub enum domainConstants {
//     minDom = 0,
//     maxDom = CONFIG_NUM_DOMAINS - 1,
//     /* To analyse branches of control flow decisions based on the number of
//      * domains without knowing their exact number, verification needs a C name
//      * to relate to higher-level specs. */
//     numDomains = CONFIG_NUM_DOMAINS
// };

#[derive(Clone)]
pub struct cap_transfer {
    pub ctReceiveRoot: cptr_t,
    pub ctReceiveIndex: cptr_t,
    pub ctReceiveDepth: word_t,
}
pub type cap_transfer_t = cap_transfer;

#[derive(Clone)]
pub enum ctLimits {
    capTransferDataSize = 3,
}

// static inline seL4_CapRights_t CONST rightsFromWord(word_t w)
// {
//     seL4_CapRights_t seL4_CapRights;

//     seL4_CapRights.words[0] = w;
//     return seL4_CapRights;
// }

// static inline word_t CONST wordFromRights(seL4_CapRights_t seL4_CapRights)
// {
//     return seL4_CapRights.words[0] & MASK(seL4_CapRightsBits);
// }

// static inline cap_transfer_t PURE capTransferFromWords(word_t *wptr)
// {
//     cap_transfer_t transfer;

//     transfer.ctReceiveRoot  = (cptr_t)wptr[0];
//     transfer.ctReceiveIndex = (cptr_t)wptr[1];
//     transfer.ctReceiveDepth = wptr[2];
//     return transfer;
// }

// static inline seL4_MessageInfo_t CONST messageInfoFromWord_raw(word_t w)
// {
//     seL4_MessageInfo_t mi;

//     mi.words[0] = w;
//     return mi;
// }

// static inline seL4_MessageInfo_t CONST messageInfoFromWord(word_t w)
// {
//     seL4_MessageInfo_t mi;
//     word_t len;

//     mi.words[0] = w;

//     len = seL4_MessageInfo_get_length(mi);
//     if (len > seL4_MsgMaxLength) {
//         mi = seL4_MessageInfo_set_length(mi, seL4_MsgMaxLength);
//     }

//     return mi;
// }

// static inline word_t CONST wordFromMessageInfo(seL4_MessageInfo_t mi)
// {
//     return mi.words[0];
// }

pub const ANSI_RESET: &str = "\033[0m";
pub const ANSI_GREEN: &str = "\033[0m\033[32m";
pub const ANSI_DARK: &str = "\033[0m\033[30;1m";
