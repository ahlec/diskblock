use crate::{
    disk::Disk,
    disk_arbitration::{
        DADiskCreateFromVolumePath, DADiskRef, DADissenterRef, DARegisterDiskMountApprovalCallback,
        DASessionCreate, DASessionRef, DASessionScheduleWithRunLoop,
    },
    dissenter::Dissenter,
};
use core_foundation::{
    base::{CFRelease, kCFAllocatorDefault},
    runloop::{CFRunLoopGetCurrent, kCFRunLoopDefaultMode},
    url::CFURLRef,
};
use std::ffi::c_void;
use std::ptr;

extern "C" fn trampoline_approval_callback<T: Fn(Disk) -> Option<Dissenter>>(
    disk: DADiskRef,
    _description: *mut c_void,
    context: *mut c_void,
) -> DADissenterRef {
    let disk = match Disk::from_ref(disk) {
        Some(disk) => disk,
        None => {
            log::error!("DADiskRef was invalid");
            return ptr::null_mut();
        }
    };

    let callback = unsafe { &(*(context as *const T)) };
    match callback(disk) {
        Some(dissenter) => dissenter.ptr,
        None => ptr::null_mut(),
    }
}

pub struct Session {
    ptr: DASessionRef,
}

impl Session {
    pub fn new() -> Option<Self> {
        let ptr = unsafe { DASessionCreate(kCFAllocatorDefault) };
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    pub fn get_disk_from_volume_path(&self, path: CFURLRef) -> Option<Disk> {
        let ptr: DADiskRef =
            unsafe { DADiskCreateFromVolumePath(kCFAllocatorDefault, self.ptr, path) };
        Disk::from_ref(ptr)
    }

    pub fn register_approval_callback<T: Fn(Disk) -> Option<Dissenter> + 'static>(
        &self,
        callback: T,
    ) -> () {
        unsafe {
            DARegisterDiskMountApprovalCallback(
                self.ptr,
                ptr::null(),
                trampoline_approval_callback::<T>,
                &callback as *const T as *mut c_void,
            );
        }
    }

    pub fn schedule(&self) -> () {
        unsafe {
            DASessionScheduleWithRunLoop(
                self.ptr,
                CFRunLoopGetCurrent() as *mut c_void,
                kCFRunLoopDefaultMode as *const c_void,
            );
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe {
            CFRelease(self.ptr as *const _);
        }
    }
}
