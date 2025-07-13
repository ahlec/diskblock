//
//  MemoryManager.swift
//  diskblock
//
//  Created by Alec Deitloff on 7/12/25.
//

import Foundation

class MemoryManager {
    private static var byteCountFormatter: ByteCountFormatter = {
        let formatter = ByteCountFormatter()
        formatter.allowedUnits = [.useMB,.useGB]
        formatter.countStyle = .binary
        formatter.isAdaptive = false
        return formatter
    }()
    
    public static func getCurrentMemoryUsage() -> String {
        guard let memoryBytes = getCurrentMemoryFootprint() else {
            return "UNKNOWN"
        }

        return byteCountFormatter.string(fromByteCount: Int64(memoryBytes))
    }
    
    private static func getCurrentMemoryFootprint() -> UInt64? {
        var info = task_vm_info_data_t()
        var count = mach_msg_type_number_t(MemoryLayout.size(ofValue: info) / MemoryLayout<natural_t>.size)

        let result = withUnsafeMutablePointer(to: &info) {
            $0.withMemoryRebound(to: integer_t.self, capacity: Int(count)) {
                task_info(mach_task_self_,
                          task_flavor_t(TASK_VM_INFO),
                          $0,
                          &count)
            }
        }

        guard result == KERN_SUCCESS else { return nil }
        return info.phys_footprint
    }
}
