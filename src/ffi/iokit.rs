use std::os::raw::{c_uint, c_void};

use core_foundation::runloop::CFRunLoopSourceRef;

#[allow(non_camel_case_types)]
pub type io_object_t = u32;

#[allow(non_camel_case_types)]
pub type io_service_t = u32;

pub type IONotificationPortRef = *mut c_void;

#[allow(non_camel_case_types)]
pub type natural_t = c_uint;

pub type IOServiceInterestCallback = Option<
    unsafe extern "C" fn(
        refcon: *mut c_void,
        service: io_service_t,
        message_type: natural_t,
        message_argument: *mut c_void,
    ),
>;

// Message type constants

// https://github.com/userlandkernel/baseband-research/blob/fb95c078d85ba51325f8e43d3f48742469b4d6ba/BBTerminal/PrivateFrameworks/IOKit/IOReturn.h#L48C9-L48C59
#[allow(non_upper_case_globals)]
const sys_iokit: u32 = crate::ffi::err_system(0x38);

#[allow(non_upper_case_globals)]
const sub_iokit_common: u32 = crate::ffi::err_sub(0);

// https://github.com/userlandkernel/baseband-research/blob/fb95c078d85ba51325f8e43d3f48742469b4d6ba/BBTerminal/PrivateFrameworks/IOKit/IOMessage.h#L44
const fn iokit_common_msg(message: u32) -> u32 {
    sys_iokit | sub_iokit_common | message
}

#[allow(non_upper_case_globals)]
pub const kIOMessageSystemWillSleep: u32 = iokit_common_msg(0x280);

#[allow(non_upper_case_globals)]
pub const kIOMessageCanSystemSleep: u32 = iokit_common_msg(0x270);

#[allow(non_upper_case_globals)]
pub const kIOMessageSystemWillPowerOn: u32 = iokit_common_msg(0x320);

#[allow(non_upper_case_globals)]
pub const kIOMessageSystemHasPoweredOn: u32 = iokit_common_msg(0x300);

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    pub unsafe fn IORegisterForSystemPower(
        refcon: *mut c_void,
        notify_port: *mut IONotificationPortRef,
        callback: IOServiceInterestCallback,
        notifier: *mut io_object_t,
    ) -> io_object_t;

    pub unsafe fn IOAllowPowerChange(root_domain_ref: io_object_t, notification_id: u64);

    pub unsafe fn IONotificationPortGetRunLoopSource(
        notify: IONotificationPortRef,
    ) -> CFRunLoopSourceRef;

    pub unsafe fn IOServiceClose(connect: io_object_t);

    pub unsafe fn IONotificationPortDestroy(notify: IONotificationPortRef);
}
