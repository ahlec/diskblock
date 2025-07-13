use core_foundation::base::kCFAllocatorDefault;
use core_foundation::runloop::{CFRunLoopGetCurrent, kCFRunLoopDefaultMode};
use std::ffi::c_void;
use std::ptr;

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
}

extern "C" fn mount_approval_callback(
    _disk: *mut c_void,
    _description: *mut c_void,
    _context: *mut c_void,
) -> *mut c_void {
    println!("Disk mount approval callback triggered");
    ptr::null_mut()
}

fn main() {
    unsafe {
        let session = DASessionCreate(kCFAllocatorDefault); //.as_concrete_TypeRef() as *const c_void);
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
            kCFRunLoopDefaultMode, //().as_concrete_TypeRef() as *const c_void,
        );

        // Keep the runloop alive
        core_foundation::runloop::CFRunLoopRun();
    }
}
