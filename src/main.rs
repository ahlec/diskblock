use core_foundation::url::CFURLRef;
use objc2_foundation::{NSFileManager, NSURL, NSVolumeEnumerationOptions};
use uuid::{Uuid, uuid};

mod disk_arbitration;

use crate::disk::Disk;
use crate::dissenter::Dissenter;
use crate::logger::init_logger;
use crate::session::Session;

mod disk;
mod dissenter;
mod logger;
mod session;

const MSI_MONITOR_UUID: Uuid = uuid!("49D00007-FF63-36B9-9D69-6B3BE16866BB");

fn is_disk_blocked(uuid: &Uuid) -> bool {
    uuid.eq(&MSI_MONITOR_UUID)
}

fn rust_mount_approval_callback(disk: Disk) -> Option<Dissenter> {
    let uuid = match disk.get_uuid() {
        Some(uuid) => uuid,
        None => {
            log::error!("Could not get UUID of mounting disk {disk}");
            return None;
        }
    };

    if !is_disk_blocked(&uuid) {
        log::info!("mounting disk {uuid} is not blocked");
        return None;
    }

    log::info!("disk {uuid} attempting to mount -- blocking");
    Some(Dissenter::new("blocked by diskblock"))
}

fn nsurl_to_cfurl_ref(nsurl: &NSURL) -> CFURLRef {
    let raw_ptr = nsurl as *const _ as *const std::ffi::c_void;
    raw_ptr.cast()
}

pub fn unmount_if_mounted(session: &Session) -> () {
    log::info!("sweeping already mounted disks");

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

        mounted_volume_urls.iter().for_each(|volume_url| {
            log::info!(
                "disk: {}",
                volume_url
                    .absoluteString()
                    .map(|x| x.to_string())
                    .unwrap_or(String::from("UNAVAILABLE"))
            );
            let path = nsurl_to_cfurl_ref(&volume_url);
            let disk = match session.get_disk_from_volume_path(path) {
                Some(disk) => disk,
                None => {
                    log::info!("  - DADisk was null");
                    return;
                }
            };

            let disk_uuid = match disk.get_uuid() {
                Some(disk_uuid) => {
                    log::info!("  - uuid: {disk_uuid}");
                    disk_uuid
                }
                None => {
                    log::info!("  - could not get uuid");
                    return;
                }
            };

            if !is_disk_blocked(&disk_uuid) {
                return;
            }

            log::info!("  - FOUND {disk_uuid}");
            disk.unmount();
        });
    }

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
