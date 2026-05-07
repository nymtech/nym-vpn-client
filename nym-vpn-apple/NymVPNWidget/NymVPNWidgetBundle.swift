import WidgetKit
import SwiftUI
import WidgetShared

@main
struct NymVPNWidgetBundle: WidgetBundle {
    var body: some Widget {
        NymVPNStatusWidget()

        if #available(iOSApplicationExtension 18.0, *) {
            NymVPNControlWidget()
        }
    }
}
