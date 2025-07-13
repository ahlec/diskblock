use core_foundation::base::{CFGetTypeID, CFType, TCFType, kCFAllocatorDefault};
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
use core_foundation::runloop::{CFRunLoopGetCurrent, kCFRunLoopDefaultMode};
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::uuid::{CFUUID, CFUUIDCreateString, CFUUIDRef};
use std::any::Any;
use std::ffi::c_void;
use std::ptr;
use uuid::{Uuid, uuid};

mod db_uuid;

#[allow(non_camel_case_types)]
type DADiskRef = *mut c_void;
#[allow(non_camel_case_types)]
type DADissenterRef = *mut c_void;

// External declaration of C function from Disk Arbitration
unsafe extern "C" {
    fn DADissenterCreate(
        allocator: *const c_void,
        status: i32,
        status_string: CFStringRef,
    ) -> DADissenterRef;
}

#[link(name = "DiskArbitration", kind = "framework")]
unsafe extern "C" {
    fn DASessionCreate(allocator: *const c_void) -> *mut c_void;

    fn DARegisterDiskMountApprovalCallback(
        session: *mut c_void,
        match_: *const c_void,
        callback: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> *mut c_void,
        context: *mut c_void,
    );

    fn DASessionScheduleWithRunLoop(
        session: *mut c_void,
        runloop: *mut c_void,
        mode: *const c_void,
    );

    fn DADiskCopyDescription(disk: DADiskRef) -> CFDictionaryRef;
}

const K_DADISK_DESCRIPTION_VOLUME_UUID_KEY: &str = "DAVolumeUUID";
const MSI_MONITOR_UUID: Uuid = uuid!("49D00007-FF63-36B9-9D69-6B3BE16866BB");

fn get_uuid_of_disk(disk: DADiskRef) -> Option<CFUUID> {
    unsafe {
        let desc_ref = DADiskCopyDescription(disk);
        if desc_ref.is_null() {
            println!("could not get description from disk {:?}", disk);
            return None;
        }

        println!("checking dictionary!");
        let description: CFDictionary<CFString, CFType> =
            CFDictionary::wrap_under_create_rule(desc_ref);
        let uuid_key = CFString::new(K_DADISK_DESCRIPTION_VOLUME_UUID_KEY);

        let value = description.find(&uuid_key)?;
        if <CFUUID as TCFType>::type_id() == CFGetTypeID(value.as_concrete_TypeRef()) {
            Some(CFUUID::wrap_under_get_rule(
                value.as_CFTypeRef() as CFUUIDRef
            ))
        } else {
            println!("UUID not found or not a CFUUID");
            None
        }
    }
}

fn is_disk_blocked(uuid: &CFUUID) -> bool {
    return uuid == MSI_MONITOR_UUID;
}

fn uuid_to_string(uuid: &CFUUID) -> String {
    unsafe {
        let cfstr_ref = CFUUIDCreateString(ptr::null(), uuid.as_concrete_TypeRef());
        let cfstr = CFString::wrap_under_create_rule(cfstr_ref);
        cfstr.to_string()
    }
}

extern "C" fn mount_approval_callback(
    disk: *mut c_void,
    _description: *mut c_void,
    _context: *mut c_void,
) -> *mut c_void {
    // unsafe {
    match get_uuid_of_disk(disk) {
        Some(uuid) => {
            // let uuid_str = CFUUIDCreateString(kCFAllocatorDefault, uuid);
            let uuid_str = uuid_to_string(&uuid);
            println!("UUID is {uuid_str}");
            if !is_disk_blocked(&uuid) {
                println!("mounting disk {} is not blocked", uuid_str);
                return ptr::null_mut(); // allow mount
            }

            println!("disk {uuid} attempting to mount -- blocking");

            // let reason = CFString::new("blocked by diskblock");
            // let dissenter = DADissenterCreate(
            //     kCFAllocatorDefault,
            //     kDAReturnExclusiveAccess as i32,
            //     reason.as_concrete_TypeRef(),
            // );

            // return dissenter;
            return ptr::null_mut();
        }
        None => {
            println!("could not get UUID of mounting disk {:?}", disk);
            return ptr::null_mut(); // allow mount
        }
    }
    // }
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

        // Keep the runloop alive
        core_foundation::runloop::CFRunLoopRun();
    }
}
