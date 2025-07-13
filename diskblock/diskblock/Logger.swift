//
//  Logger.swift
//  diskblock
//
//  Created by Alec Deitloff on 7/12/25.
//

import Foundation

class Logger {
    private static var dateFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd HH:mm:ss.SSS "
        return formatter
    }()
    
    public static func log(_ message: String) {
        let now = dateFormatter.string(from: Date())
        print("[\(now)] \(message)")
    }
}
