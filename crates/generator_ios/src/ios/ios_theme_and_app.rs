fn ios_theme(design_config: &DesignConfig) -> String {
    swift_theme_module(design_config)
}

fn ios_responsive() -> String {
    r#"import SwiftUI

enum DoweResponsiveModule {
    static let generated = true
}
"#
    .to_string()
}

fn ios_app() -> String {
    r#"import SwiftUI
import UIKit
import UserNotifications

final class DoweNotificationDelegate: NSObject, UIApplicationDelegate, UNUserNotificationCenterDelegate {
    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]? = nil) -> Bool {
        UNUserNotificationCenter.current().delegate = self
        return true
    }

    func userNotificationCenter(_ center: UNUserNotificationCenter, didReceive response: UNNotificationResponse, withCompletionHandler completionHandler: @escaping () -> Void) {
        if let route = response.notification.request.content.userInfo["route"] as? String,
           route.hasPrefix("/"),
           !route.contains("//"),
           let url = URL(string: "dowe-dev://generated\(route)") {
            DispatchQueue.main.async { UIApplication.shared.open(url) }
        }
        completionHandler()
    }
}

@main
struct DoweIosApp: App {
    @UIApplicationDelegateAdaptor(DoweNotificationDelegate.self) private var notificationDelegate

    var body: some Scene {
        WindowGroup {
            DoweRootView()
        }
    }
}
"#
    .to_string()
}

