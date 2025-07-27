use uuid::{Uuid, uuid};

use crate::disk::Disk;
use crate::dissenter::Dissenter;
use crate::logger::init_logger;
use crate::session::Session;
use crate::system_events::{PowerEvent, listen_for_power_events};

mod disk;
mod dissenter;
mod ffi;
mod logger;
mod session;
mod system_events;

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
    let mut did_unmount = false;
    for disk in session.get_mounted_disks() {
        if !is_disk_blocked(&disk.uuid) {
            continue;
        }

        log::info!("Unmounting already mounted disk: {disk}");
        disk.unmount();
        did_unmount = true;
    }

    if !did_unmount {
        log::info!("Swept mounted disks, no blocked disks found mounted");
    }
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

    let subscription = listen_for_power_events(|event| {
        let msg = match event {
            PowerEvent::WillPowerOn => "WILL POWER ON",
            PowerEvent::WillSleep => "WILL SLEEP",
            _ => {
                // Ignore other events
                return;
            }
        };

        log::info!("power event: {msg}");
        unmount_if_mounted(&session);
    });
    log::info!("Subscribed to power events");

    unsafe {
        core_foundation::runloop::CFRunLoopRun();
    }

    drop(subscription);

    Ok(())
}
