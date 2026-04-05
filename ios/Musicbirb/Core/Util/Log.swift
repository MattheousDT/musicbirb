import Foundation
import os

struct Log {
	// This allows us to filter specifically for our app and ignore system noise
	private static let subsystem = Bundle.main.bundleIdentifier ?? "com.musicbirb.Musicbirb"

	static let app = Logger(subsystem: subsystem, category: "App")
	static let audio = Logger(subsystem: subsystem, category: "Audio")
	static let rust = Logger(subsystem: subsystem, category: "Rust")
}
