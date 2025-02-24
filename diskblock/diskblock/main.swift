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

func create_uuid(_ str: String) -> CFUUIDBytes {
    let uuid = CFUUIDCreateFromString(kCFAllocatorDefault, str as CFString)
    return CFUUIDGetUUIDBytes(uuid)
}

func are_uuid_equal(_ a: CFUUIDBytes, _ b: CFUUIDBytes) -> Bool {
    // TODO
    return a.byte0 == b.byte0
}

let MSI_MONITOR_UUID = create_uuid("49D00007-FF63-36B9-9D69-6B3BE16866BB")

func allow_mount(disk: DADisk, context: UnsafeMutableRawPointer?) -> Unmanaged<
    DADissenter
>? {
    guard let description = DADiskCopyDescription(disk) as? [CFString: Any]
    else {
        print("could not get description from disk")
        return nil
    }

    guard let uuid = description[kDADiskDescriptionVolumeUUIDKey] as! CFUUID?
    else {
        print("could not get UUID from description")
        return nil
    }

    let uuidBytes = CFUUIDGetUUIDBytes(uuid)
    if are_uuid_equal(uuidBytes, MSI_MONITOR_UUID) {
        print("NO NAME MOUNTED")
        let dissenter = DADissenterCreate(
            kCFAllocatorDefault, Int32(kDAReturnExclusiveAccess),
            "It's mine!" as CFString)
        return Unmanaged.passRetained(dissenter)
    }

    print("something else mounted: \(uuid)")
    //        int allow = 0;
    //
    //        if (allow) {
    //                /* Return NULL to allow */
    //                fprintf(stderr, "allow_mount: allowing mount.\n");
    //                return NULL;
    //        } else {
    //                /* Return a dissenter to deny */
    //                fprintf(stderr, "allow_mount: refusing mount.\n");
    //                return DADissenterCreate(
    //                        kCFAllocatorDefault, kDAReturnExclusiveAccess,
    //                        CFSTR("It's mine!"));
    //        }
    return nil
}

func main() {
    guard let session = DASessionCreate(kCFAllocatorDefault) else {
        print("couldn't allocate session")
        return
    }

    print("allocated")

    DARegisterDiskMountApprovalCallback(
        session,
        nil, /* Match all disks */
        allow_mount,
        nil) /* No context */

    DASessionScheduleWithRunLoop(
        session, CFRunLoopGetCurrent(), CFRunLoopMode.defaultMode.rawValue)

    let app = NSApplication.shared
    //    app.delegate = FSAppDelegate.init()
    app.activate(ignoringOtherApps: true)
    app.run()
}

main()
