use core_foundation::{
    base::{TCFType, kCFAllocatorDefault},
    string::CFString,
};

use crate::ffi::disk_arbitration::{DADissenterCreate, DADissenterRef, kDAReturnExclusiveAccess};

pub struct Dissenter {
    pub ptr: DADissenterRef,
}

impl Dissenter {
    pub fn new(reason: &str) -> Dissenter {
        let reason = CFString::new(reason);
        let ptr: DADissenterRef = unsafe {
            DADissenterCreate(
                kCFAllocatorDefault,
                kDAReturnExclusiveAccess as i32,
                reason.as_concrete_TypeRef(),
            )
        };

        if ptr.is_null() {
            panic!("DADissenterCreate returned null");
        }

        Self { ptr }
    }
}
