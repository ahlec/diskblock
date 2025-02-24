//
//  main.swift
//  diskblock
//
//  Created by Alec Deitloff on 2/18/25.
//

import Cocoa
import CoreFoundation
import DiskArbitration
import Foundation

func make_cfuuid(_ str: String) -> CFUUID {
    return CFUUIDCreateFromString(kCFAllocatorDefault, str as CFString)
}

let MSI_MONITOR_UUID = make_cfuuid("49D00007-FF63-36B9-9D69-6B3BE16866BB")

func get_uuid_of_disk(_ disk: DADisk) -> CFUUID? {
    guard let description = DADiskCopyDescription(disk) as? [CFString: Any]
    else {
        print("could not get description from disk \(disk)")
        return nil
    }

    return description[kDADiskDescriptionVolumeUUIDKey] as! CFUUID?
}

func is_disk_blocked(_ uuid: CFUUID) -> Bool {
    return uuid == MSI_MONITOR_UUID
}

func unmount_if_mounted(_ session: DASession) {
    print("sweeping already mounted disks")
    guard
        let mountedVolumeURLs = FileManager.default.mountedVolumeURLs(
            includingResourceValuesForKeys: nil)
    else {
        print("unable to get mounted volumes")
        return
    }

    for volumeURL in mountedVolumeURLs {
        if let disk = DADiskCreateFromVolumePath(
            kCFAllocatorDefault, session, volumeURL as CFURL)
        {
            if let uuid = get_uuid_of_disk(disk) {
                if is_disk_blocked(uuid) {
                    print("FOUND \(uuid) as disk \(disk)")
                    DADiskUnmount(
                        disk, UInt32(kDADiskUnmountOptionDefault), nil, nil)
                }
            }
        }
    }

    print("finished sweeping mounted disks")
}

func mount_approval_callback(disk: DADisk, context: UnsafeMutableRawPointer?)
    -> Unmanaged<
        DADissenter
    >?
{
    guard let uuid = get_uuid_of_disk(disk) else {
        print("could not get UUID of mounting disk \(disk)")
        return nil
    }

    if !is_disk_blocked(uuid) {
        print("mounting disk \(uuid) is not blocked: \(disk)")
        return nil
    }

    print("disk \(uuid) attempting to mount -- blocking")
    let dissenter = DADissenterCreate(
        kCFAllocatorDefault, Int32(kDAReturnExclusiveAccess),
        "blocked by diskblock" as CFString)
    return Unmanaged.passRetained(dissenter)
}

func main() {
    guard let session = DASessionCreate(kCFAllocatorDefault) else {
        print("couldn't allocate session")
        return
    }

    unmount_if_mounted(session)

    print("registering callback")
    DARegisterDiskMountApprovalCallback(
        session,
        nil, /* Match all disks */
        mount_approval_callback,
        nil)

    DASessionScheduleWithRunLoop(
        session, CFRunLoopGetCurrent(), CFRunLoopMode.defaultMode.rawValue)

    let app = NSApplication.shared
    app.activate(ignoringOtherApps: true)
    app.run()
}

main()
