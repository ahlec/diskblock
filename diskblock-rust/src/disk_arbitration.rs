use core_foundation::dictionary::CFDictionaryRef;
use core_foundation::string::CFStringRef;
use std::ffi::c_void;

#[allow(non_camel_case_types)]
pub type DADiskRef = *mut c_void;
#[allow(non_camel_case_types)]
pub type DADissenterRef = *mut c_void;

#[link(name = "DiskArbitration", kind = "framework")]
unsafe extern "C" {
    pub fn DASessionCreate(allocator: *const c_void) -> *mut c_void;

    pub fn DARegisterDiskMountApprovalCallback(
        session: *mut c_void,
        match_: *const c_void,
        callback: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void,
        context: *mut c_void,
    );

    pub fn DASessionScheduleWithRunLoop(
        session: *mut c_void,
        runloop: *mut c_void,
        mode: *const c_void,
    );

    pub fn DADiskCopyDescription(disk: DADiskRef) -> CFDictionaryRef;

    pub fn DADissenterCreate(
        allocator: *const c_void,
        status: i32,
        status_string: CFStringRef,
    ) -> DADissenterRef;
}
