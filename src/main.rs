use core_foundation::url::CFURLRef;
use objc2_foundation::{NSFileManager, NSURL, NSVolumeEnumerationOptions};
use uuid::{Uuid, uuid};

mod disk_arbitration;

use crate::disk::Disk;
use crate::dissenter::Dissenter;
use crate::session::Session;

mod disk;
mod dissenter;
mod session;

const MSI_MONITOR_UUID: Uuid = uuid!("49D00007-FF63-36B9-9D69-6B3BE16866BB");

fn is_disk_blocked(uuid: &Uuid) -> bool {
    uuid.eq(&MSI_MONITOR_UUID)
}

fn rust_mount_approval_callback(disk: Disk) -> Option<Dissenter> {
    let uuid = match disk.get_uuid() {
        Some(uuid) => uuid,
        None => {
            println!("Could not get UUID of mounting disk {disk}");
            return None;
        }
    };

    if !is_disk_blocked(&uuid) {
        println!("mounting disk {uuid} is not blocked");
        return None;
    }

    println!("disk {uuid} attempting to mount -- blocking");
    Some(Dissenter::new("blocked by diskblock"))
}

fn nsurl_to_cfurl_ref(nsurl: &NSURL) -> CFURLRef {
    let raw_ptr = nsurl as *const _ as *const std::ffi::c_void;
    raw_ptr.cast()
}

pub fn unmount_if_mounted(session: &Session) -> () {
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
            let disk = match session.get_disk_from_volume_path(path) {
                Some(disk) => disk,
                None => {
                    println!("  - DADisk was null");
                    return;
                }
            };

            let disk_uuid = match disk.get_uuid() {
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
            disk.unmount();
        });
    }

    println!("finished sweeping mounted disks");
}

fn main() -> Result<(), ()> {
    let session = match Session::new() {
        Some(session) => session,
        None => {
            println!("couldn't allocate session");
            return Err(());
        }
    };

    session.register_approval_callback(rust_mount_approval_callback);
    println!("Callback registered");

    session.schedule();
    println!("Session scheduled");

    unmount_if_mounted(&session);

    unsafe {
        core_foundation::runloop::CFRunLoopRun();
    }

    Ok(())
}
