fn ios_notifications() -> String {
    r#"import Foundation
import UIKit
import UserNotifications

enum DoweNotifications {
    static func requestPermission() async -> Bool {
        let granted = (try? await UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .badge, .sound])) ?? false
        if granted {
            await MainActor.run { UIApplication.shared.registerForRemoteNotifications() }
        }
        return granted
    }

    static func show(id: String, title: String, body: String, route: String? = nil) async throws {
        guard !id.isEmpty, !title.isEmpty, !body.isEmpty else { throw DoweNotificationError.invalidPayload }
        if let route, (!route.hasPrefix("/") || route.contains("//")) { throw DoweNotificationError.invalidRoute }
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        content.sound = .default
        if let route { content.userInfo = ["route": route] }
        let request = UNNotificationRequest(
            identifier: id,
            content: content,
            trigger: UNTimeIntervalNotificationTrigger(timeInterval: 0.1, repeats: false)
        )
        try await UNUserNotificationCenter.current().add(request)
    }
}

enum DoweNotificationError: Error {
    case invalidPayload
    case invalidRoute
}
"#
    .to_string()
}

fn ios_entitlements() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>aps-environment</key>
    <string>$(APS_ENVIRONMENT)</string>
</dict>
</plist>
"#
    .to_string()
}

