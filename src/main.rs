use core_foundation::base::{CFGetTypeID, CFType, TCFType, kCFAllocatorDefault};
use core_foundation::dictionary::CFDictionary;
use core_foundation::runloop::{CFRunLoopGetCurrent, kCFRunLoopDefaultMode};
use core_foundation::string::CFString;
use core_foundation::url::CFURLRef;
use core_foundation::uuid::{CFUUID, CFUUIDGetUUIDBytes, CFUUIDRef};
use objc2_foundation::{NSFileManager, NSURL, NSVolumeEnumerationOptions};
use std::ffi::c_void;
use std::ptr;
use uuid::{Uuid, uuid};

mod disk_arbitration;
use disk_arbitration::*;

const MSI_MONITOR_UUID: Uuid = uuid!("49D00007-FF63-36B9-9D69-6B3BE16866BB");

fn get_uuid_of_disk(disk: DADiskRef) -> Option<Uuid> {
    unsafe {
        let desc_ref = DADiskCopyDescription(disk);
        if desc_ref.is_null() {
            println!("could not get description from disk {:?}", disk);
            return None;
        }

        let description: CFDictionary<CFString, CFType> =
            CFDictionary::wrap_under_create_rule(desc_ref);
        let uuid_key = CFString::new(kDADiskDescriptionVolumeUUIDKey);

        let value = description.find(&uuid_key)?;

        if CFUUID::type_id() != CFGetTypeID(value.as_concrete_TypeRef()) {
            println!("UUID not found or not a CFUUID");
            return None;
        }

        let cf_uuid_ref = value.as_CFTypeRef() as CFUUIDRef;
        let bytes = CFUUIDGetUUIDBytes(cf_uuid_ref);
        Some(Uuid::from_bytes([
            bytes.byte0,
            bytes.byte1,
            bytes.byte2,
            bytes.byte3,
            bytes.byte4,
            bytes.byte5,
            bytes.byte6,
            bytes.byte7,
            bytes.byte8,
            bytes.byte9,
            bytes.byte10,
            bytes.byte11,
            bytes.byte12,
            bytes.byte13,
            bytes.byte14,
            bytes.byte15,
        ]))
    }
}

fn is_disk_blocked(uuid: &Uuid) -> bool {
    uuid.eq(&MSI_MONITOR_UUID)
}

extern "C" fn mount_approval_callback(
    disk: *mut c_void,
    _description: *mut c_void,
    _context: *mut c_void,
) -> *mut c_void {
    match get_uuid_of_disk(disk) {
        Some(uuid) => {
            if !is_disk_blocked(&uuid) {
                println!("mounting disk {uuid} is not blocked");
                return ptr::null_mut(); // allow mount
            }

            println!("disk {uuid} attempting to mount -- blocking");

            let reason = CFString::new("blocked by diskblock");
            unsafe {
                let dissenter = DADissenterCreate(
                    kCFAllocatorDefault,
                    kDAReturnExclusiveAccess as i32,
                    reason.as_concrete_TypeRef(),
                );

                return dissenter;
            }
        }
        None => {
            println!("could not get UUID of mounting disk {:?}", disk);
            return ptr::null_mut(); // allow mount
        }
    }
}

fn nsurl_to_cfurl_ref(nsurl: &NSURL) -> CFURLRef {
    let raw_ptr = nsurl as *const _ as *const std::ffi::c_void;
    raw_ptr.cast()
}

pub fn unmount_if_mounted(session: DASessionRef) -> () {
    println!("sweeping already mounted disks");

    unsafe {
        let mounted_volume_urls = NSFileManager::defaultManager()
            .mountedVolumeURLsIncludingResourceValuesForKeys_options(
                None,
                NSVolumeEnumerationOptions::empty(),
            )
            .unwrap_or_else(|| {
                println!("unable to get mounted volumes");
                return Default::default();
            });

        mounted_volume_urls.iter().for_each(|volume_url| {
            println!(
                "disk: {}",
                volume_url
                    .absoluteString()
                    .map(|x| x.to_string())
                    .unwrap_or(String::from("UNAVAILABLE"))
            );
            let path = nsurl_to_cfurl_ref(&volume_url);
            let disk = DADiskCreateFromVolumePath(kCFAllocatorDefault, session, path);
            if disk.is_null() {
                println!("  - DADisk was null");
                return;
            }

            let disk_uuid = match get_uuid_of_disk(disk) {
                Some(disk_uuid) => {
                    println!("  - uuid: {disk_uuid}");
                    disk_uuid
                }
                None => {
                    println!("  - could not get uuid");
                    return;
                }
            };

            if !is_disk_blocked(&disk_uuid) {
                return;
            }

            println!("  - FOUND {disk_uuid}");
            DADiskUnmount(
                disk,
                kDADiskUnmountOptionDefault,
                ptr::null_mut(),
                ptr::null_mut(),
            );
        });
    }

    println!("finished sweeping mounted disks");
}

fn main() {
    unsafe {
        let session = DASessionCreate(kCFAllocatorDefault);
        if session.is_null() {
            println!("couldn't allocate session");
            return;
        }

        println!("registering callback");

        // Match all disks by passing null
        DARegisterDiskMountApprovalCallback(
            session,
            ptr::null(),
            mount_approval_callback,
            ptr::null_mut(),
        );

        // Schedule with current run loop
        DASessionScheduleWithRunLoop(
            session,
            CFRunLoopGetCurrent() as *mut c_void,
            kCFRunLoopDefaultMode as *const c_void,
        );

        unmount_if_mounted(session);

        // Keep the runloop alive
        core_foundation::runloop::CFRunLoopRun();
    }
}
