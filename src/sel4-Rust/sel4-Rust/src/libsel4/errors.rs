//to be done
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

/*
Including contents from:
sel4/errors.h
 */

pub enum seL4_Error {
    seL4_NoError = 0,
    seL4_InvalidArgument,
    seL4_InvalidCapability,
    seL4_IllegalOperation,
    seL4_RangeError,
    seL4_AlignmentError,
    seL4_FailedLookup,
    seL4_TruncatedMessage,
    seL4_DeleteFirst,
    seL4_RevokeFirst,
    seL4_NotEnoughMemory,

    /* This should always be the last item in the list
     * so it gives a count of the number of errors in the
     * enum.
     */
    seL4_NumErrors,
}
