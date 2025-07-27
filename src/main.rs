use uuid::{Uuid, uuid};

use crate::disk::Disk;
use crate::dissenter::Dissenter;
use crate::logger::init_logger;
use crate::session::Session;

mod disk;
mod dissenter;
mod ffi;
mod logger;
mod session;

const MSI_MONITOR_UUID: Uuid = uuid!("49D00007-FF63-36B9-9D69-6B3BE16866BB");

fn is_disk_blocked(uuid: &Uuid) -> bool {
    uuid.eq(&MSI_MONITOR_UUID)
}

fn rust_mount_approval_callback(disk: Disk) -> Option<Dissenter> {
    if !is_disk_blocked(&disk.uuid) {
        log::info!("mounting disk {} is not blocked", disk.uuid);
        return None;
    }

    log::info!("disk {} attempting to mount -- blocking", disk.uuid);
    Some(Dissenter::new("blocked by diskblock"))
}

pub fn unmount_if_mounted(session: &Session) -> () {
    log::info!("sweeping already mounted disks");

    session.get_mounted_disks().for_each(|disk| {
        log::info!("- {disk}");
        if !is_disk_blocked(&disk.uuid) {
            return;
        }

        log::info!("  -> UNMOUNTING");
        disk.unmount();
    });

    log::info!("finished sweeping mounted disks");
}

fn main() -> Result<(), ()> {
    init_logger();

    log::info!("============================================================");

    let session = match Session::new() {
        Some(session) => session,
        None => {
            log::error!("couldn't allocate session");
            return Err(());
        }
    };

    session.register_approval_callback(rust_mount_approval_callback);
    log::info!("Callback registered");

    session.schedule();
    log::info!("Session scheduled");

    unmount_if_mounted(&session);

    unsafe {
        core_foundation::runloop::CFRunLoopRun();
    }

    Ok(())
}
