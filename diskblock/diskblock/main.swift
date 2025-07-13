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
        Logger.log("could not get description from disk \(disk)")
        return nil
    }

    return description[kDADiskDescriptionVolumeUUIDKey] as! CFUUID?
}

func is_disk_blocked(_ uuid: CFUUID) -> Bool {
    return uuid == MSI_MONITOR_UUID
}

func unmount_if_mounted(_ session: DASession) {
    Logger.log("sweeping already mounted disks")
    guard
        let mountedVolumeURLs = FileManager.default.mountedVolumeURLs(
            includingResourceValuesForKeys: nil)
    else {
        Logger.log("unable to get mounted volumes")
        return
    }

    for volumeURL in mountedVolumeURLs {
        if let disk = DADiskCreateFromVolumePath(
            kCFAllocatorDefault, session, volumeURL as CFURL)
        {
            if let uuid = get_uuid_of_disk(disk) {
                if is_disk_blocked(uuid) {
                    Logger.log("FOUND \(uuid) as disk \(disk)")
                    DADiskUnmount(
                        disk, UInt32(kDADiskUnmountOptionDefault), nil, nil)
                }
            }
        }
    }

    Logger.log("finished sweeping mounted disks")
}

func mount_approval_callback(disk: DADisk, context: UnsafeMutableRawPointer?)
    -> Unmanaged<
        DADissenter
    >?
{
    guard let uuid = get_uuid_of_disk(disk) else {
        Logger.log("could not get UUID of mounting disk \(disk)")
        return nil
    }

    if !is_disk_blocked(uuid) {
        Logger.log("mounting disk \(uuid) is not blocked: \(disk)")
        return nil
    }

    Logger.log("disk \(uuid) attempting to mount -- blocking")
    let dissenter = DADissenterCreate(
        kCFAllocatorDefault, Int32(kDAReturnExclusiveAccess),
        "blocked by diskblock" as CFString)
    return Unmanaged.passRetained(dissenter)
}

func logMemoryUsage() -> Void {
    Logger.log("memory: \(MemoryManager.getCurrentMemoryUsage())")
}

func main() {
    Logger.log("pid: \(getpid())")

    logMemoryUsage()
    Timer.scheduledTimer(withTimeInterval: 60.0, repeats: true) { _ in
        logMemoryUsage()
    }
    
    guard let session = DASessionCreate(kCFAllocatorDefault) else {
        Logger.log("couldn't allocate session")
        return
    }

    unmount_if_mounted(session)

    Logger.log("registering callback")
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
