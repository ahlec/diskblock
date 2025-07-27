use core_foundation::dictionary::CFDictionaryRef;
use core_foundation::string::CFStringRef;
use core_foundation::url::CFURLRef;
use std::ffi::c_void;

// https://codebrowser.dev/glibc/glibc/mach/mach/error.h.html
const fn err_system(x: u32) -> u32 {
    (x & 0x3f) << 26
}

const fn err_sub(x: u32) -> u32 {
    (x & 0xfff) << 14
}

// https://codebrowser.dev/glibc/glibc/mach/mach/error.h.html
#[allow(non_upper_case_globals)]
const err_local: u32 = err_system(0x3e); /* user defined errors */

// https://github.com/phracker/MacOSX-SDKs/blob/041600eda65c6a668f66cb7d56b7d1da3e8bcc93/MacOSX10.9.sdk/System/Library/Frameworks/DiskArbitration.framework/Versions/A/Headers/DADissenter.h#L34
#[allow(non_upper_case_globals)]
const err_local_diskarbitration: u32 = err_sub(0x368);

pub type DADiskRef = *mut c_void;

pub type DADissenterRef = *mut c_void;

pub type DASessionRef = *mut c_void;

// https://github.com/phracker/MacOSX-SDKs/blob/041600eda65c6a668f66cb7d56b7d1da3e8bcc93/MacOSX10.9.sdk/System/Library/Frameworks/DiskArbitration.framework/Versions/A/Headers/DADissenter.h#L47C5-L47C29
#[allow(non_upper_case_globals)]
pub const kDAReturnExclusiveAccess: u32 = err_local | err_local_diskarbitration | 0x04;

#[allow(non_upper_case_globals)]
pub const kDADiskDescriptionVolumeNameKey: &str = "DAVolumeName";

#[allow(non_upper_case_globals)]
pub const kDADiskDescriptionVolumePathKey: &str = "DAVolumePath";

#[allow(non_upper_case_globals)]
pub const kDADiskDescriptionVolumeUUIDKey: &str = "DAVolumeUUID";

#[allow(non_upper_case_globals)]
pub const kDADiskUnmountOptionDefault: u32 = 0;

#[link(name = "DiskArbitration", kind = "framework")]
unsafe extern "C" {
    pub fn DASessionCreate(allocator: *const c_void) -> DASessionRef;

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

    pub fn DADiskCreateFromVolumePath(
        allocator: *const c_void,
        session: DASessionRef,
        path: CFURLRef,
    ) -> DADiskRef;

    pub fn DADiskUnmount(
        disk: DADiskRef,
        options: u32,
        callback: *mut c_void,
        context: *mut c_void,
    );
}
