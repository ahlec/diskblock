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
use objc2_foundation::{NSFileManager, NSURL, NSVolumeEnumerationOptions};
use std::ptr;
use std::{ffi::c_void, path::PathBuf};

extern "C" fn trampoline_approval_callback<T: Fn(Disk) -> Option<Dissenter>>(
    disk: DADiskRef,
    _description: *mut c_void,
    context: *mut c_void,
) -> DADissenterRef {
    let disk = match Disk::from_ref(disk, None) {
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

fn nsurl_to_cfurl_ref(nsurl: &NSURL) -> CFURLRef {
    let raw_ptr = nsurl as *const _ as *const std::ffi::c_void;
    raw_ptr.cast()
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

    pub fn get_mounted_disks(&self) -> impl Iterator<Item = Disk> {
        unsafe {
            let mounted_volume_urls = NSFileManager::defaultManager()
                .mountedVolumeURLsIncludingResourceValuesForKeys_options(
                    None,
                    NSVolumeEnumerationOptions::empty(),
                )
                .unwrap_or_else(|| {
                    log::error!("unable to get mounted volumes");
                    return Default::default();
                });

            mounted_volume_urls.into_iter().filter_map(|volume_url| {
                let path = nsurl_to_cfurl_ref(&volume_url);
                let ptr: DADiskRef =
                    DADiskCreateFromVolumePath(kCFAllocatorDefault, self.ptr, path);
                let path = volume_url
                    .absoluteString()
                    .map(|str| PathBuf::from(str.to_string()));
                Disk::from_ref(ptr, path)
            })
        }
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
