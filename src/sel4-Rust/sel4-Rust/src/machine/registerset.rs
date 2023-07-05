//to be done

#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::types::basic_types::*;

/*
Including contents from
1. machine/registerset.h
2. arch/machine/registerset.h
 */


// 2. from arch/machine/registerset.h
#[derive(Clone)]
pub struct user_context {
    pub registers:[word_t;35]
}
pub type user_context_t=user_context;

