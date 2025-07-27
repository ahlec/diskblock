use core_foundation::runloop::CFRunLoopAddSource;
use core_foundation::runloop::CFRunLoopSourceRef;
use core_foundation::runloop::kCFRunLoopDefaultMode;

use std::os::raw::c_void;
use std::ptr;

use crate::ffi::iokit::IONotificationPortDestroy;
use crate::ffi::iokit::IOServiceClose;
use crate::ffi::iokit::kIOMessageSystemWillPowerOn;
use crate::ffi::iokit::{
    IOAllowPowerChange, IONotificationPortGetRunLoopSource, IONotificationPortRef,
    IORegisterForSystemPower, io_service_t, kIOMessageCanSystemSleep, kIOMessageSystemHasPoweredOn,
    kIOMessageSystemWillSleep, natural_t,
};

struct PowerEventCallbackInfo<'a> {
    root_port: Option<u32>,
    callback: Box<dyn Fn(PowerEvent) -> () + 'a>,
}

pub enum PowerEvent {
    WillSleep,
    CanSleep,
    WillPowerOn,
    HasPoweredOn,
}

extern "C" fn trampoline_power_callback<'a>(
    refcon: *mut c_void,
    _service: io_service_t,
    message_type: natural_t,
    message_argument: *mut c_void,
) {
    let callback_info = unsafe { &mut *(refcon as *mut PowerEventCallbackInfo<'_>) };
    let root_port = match callback_info.root_port {
        Some(root_port) => root_port,
        None => {
            panic!("Inside trampoline_power_callback without root_port");
        }
    };

    let callback = &callback_info.callback;

    // Choosing to not do `match messageType => PowerEvent` and instead call them independently
    // so that we can guarantee callback is called prior to IOAllowerPowerChange
    match message_type {
        #[allow(non_upper_case_globals)]
        kIOMessageSystemWillSleep => {
            callback(PowerEvent::WillSleep);
            unsafe {
                IOAllowPowerChange(root_port, message_argument as u64);
            }
        }
        #[allow(non_upper_case_globals)]
        kIOMessageCanSystemSleep => {
            callback(PowerEvent::CanSleep);
            unsafe {
                IOAllowPowerChange(root_port, message_argument as u64);
            }
        }
        #[allow(non_upper_case_globals)]
        kIOMessageSystemWillPowerOn => callback(PowerEvent::WillPowerOn),
        #[allow(non_upper_case_globals)]
        kIOMessageSystemHasPoweredOn => callback(PowerEvent::HasPoweredOn),
        _ => {
            println!("Unknown power message: {message_type}");
        }
    }
}

pub struct PowerEventSubscription<'a> {
    callback_info_ptr: *mut PowerEventCallbackInfo<'a>,
    notify_port: IONotificationPortRef,
}

/// Adds a listener to the power events from the system, invoking the callback during
/// events related to sleeping/waking.
///
/// The listener will be subscribed for as long as the returned object is live. When
/// the PowerEventSubscription is dropped, the event listener will be unsubscribed and
/// all memory will be freed.
pub fn listen_for_power_events<'a, T: Fn(PowerEvent) -> () + 'a>(
    callback: T,
) -> PowerEventSubscription<'a> {
    let mut notify_port: IONotificationPortRef = ptr::null_mut();
    let mut notifier = 0;

    let callback_info: PowerEventCallbackInfo<'a> = PowerEventCallbackInfo {
        root_port: None,
        callback: Box::new(callback),
    };
    let callback_info = Box::new(callback_info);
    let callback_info_ptr = Box::into_raw(callback_info);

    let root_port = unsafe {
        IORegisterForSystemPower(
            callback_info_ptr as *mut c_void,
            &mut notify_port,
            Some(trampoline_power_callback),
            &mut notifier,
        )
    };

    if root_port == 0 {
        panic!("IORegisterForSystemPower failed");
    }

    unsafe {
        (*callback_info_ptr).root_port = Some(root_port);
    }

    let run_loop_source: CFRunLoopSourceRef =
        unsafe { IONotificationPortGetRunLoopSource(notify_port) };

    unsafe {
        CFRunLoopAddSource(
            core_foundation::runloop::CFRunLoopGetCurrent(),
            run_loop_source,
            kCFRunLoopDefaultMode,
        )
    }

    PowerEventSubscription {
        callback_info_ptr,
        notify_port,
    }
}

impl Drop for PowerEventSubscription<'_> {
    fn drop(&mut self) {
        unsafe {
            IONotificationPortDestroy(self.notify_port);

            let info = Box::from_raw(self.callback_info_ptr as *mut PowerEventCallbackInfo<'_>);
            if let Some(root_port) = info.root_port {
                IOServiceClose(root_port);
            }
        }
    }
}
