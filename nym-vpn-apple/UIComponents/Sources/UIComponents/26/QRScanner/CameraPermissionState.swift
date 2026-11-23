#if os(iOS)
import AVFoundation

enum CameraPermissionState: Equatable {
    case granted
    case notDetermined
    case denied

    static var current: CameraPermissionState {
        switch AVCaptureDevice.authorizationStatus(for: .video) {
        case .authorized:
            .granted
        case .notDetermined:
            .notDetermined
        case .denied, .restricted:
            .denied
        @unknown default:
            .denied
        }
    }

    static func request() async -> CameraPermissionState {
        await AVCaptureDevice.requestAccess(for: .video) ? .granted : .denied
    }
}
#endif
