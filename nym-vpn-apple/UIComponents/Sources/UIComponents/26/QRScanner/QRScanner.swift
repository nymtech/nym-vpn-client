#if os(iOS)
import SwiftUI
import ImpactGenerator
import Theme

/// Returns only the scanned content; the validator decides whether it's accepted.
public struct QRScanner: View {
    private let secondaryAction: QRScanAction?
    private let isPaused: Bool
    private let onResult: (QRScanResult) -> Void
    private let onClose: () -> Void

    @State private var viewModel: QRScannerViewModel
    @State private var permission = CameraPermissionState.current
    @State private var didRequestPermission = false
    @State private var hasCameraError = false
    @State private var isTorchOn = false
    @Environment(\.scenePhase)
    private var scenePhase
    @Environment(\.openURL)
    private var openURL

    /// False without a camera, e.g. in the Simulator.
    public static var isSupported: Bool {
        QRCameraPreview.isSupported
    }

    public init(
        validator: QRScanValidator,
        secondaryAction: QRScanAction? = nil,
        isPaused: Bool = false,
        onResult: @escaping (QRScanResult) -> Void,
        onClose: @escaping () -> Void
    ) {
        self.secondaryAction = secondaryAction
        self.isPaused = isPaused
        self.onResult = onResult
        self.onClose = onClose
        _viewModel = State(wrappedValue: QRScannerViewModel(validator: validator))
    }

    public var body: some View {
        ZStack(alignment: .top) {
            Color.black
                .ignoresSafeArea()
            content
            topBar
        }
        .environment(\.colorScheme, .dark)
        .task {
            await requestPermissionIfNeeded()
        }
        .onChange(of: scenePhase) { _, phase in
            // Picks up a grant or revoke made in system settings.
            guard phase == .active else { return }
            permission = CameraPermissionState.current
        }
        .onChange(of: viewModel.result) { _, result in
            guard let result else { return }
            ImpactGenerator.shared.success()
            onResult(result)
        }
        .onChange(of: viewModel.invalidMessage) { _, message in
            guard let message else { return }
            AccessibilityNotification.Announcement(message).post()
        }
        .onChange(of: viewModel.isValidating) { _, isValidating in
            guard isValidating else { return }
            AccessibilityNotification.Announcement("qrScanner.checking".localizedString).post()
        }
        .onChange(of: isPaused) { _, paused in
            if paused { setTorch(on: false) }
        }
        .onDisappear {
            setTorch(on: false)
            viewModel.cancelPendingWork()
        }
    }
}

private extension QRScanner {
    var isCameraActive: Bool {
        permission == .granted && !hasCameraError
    }

    @ViewBuilder
    var content: some View {
        if hasCameraError {
            QRScannerMessageView(
                systemImageName: "exclamationmark.triangle",
                title: "qrScanner.cameraError.title".localizedString,
                message: "qrScanner.cameraError.body".localizedString,
                secondaryAction: secondaryAction
            )
        } else if permission == .granted {
            ZStack {
                QRCameraPreview(
                    isScanning: !isPaused && viewModel.result == nil,
                    onCodeScan: { viewModel.onCodeScan($0) },
                    onCameraError: { hasCameraError = true }
                )
                .ignoresSafeArea()
                .accessibilityHidden(true)
                if isPaused {
                    Rectangle()
                        .fill(.ultraThinMaterial)
                        .ignoresSafeArea()
                        .accessibilityHidden(true)
                } else {
                    QRScannerOverlay(
                        invalidMessage: viewModel.invalidMessage,
                        isValidating: viewModel.isValidating,
                        secondaryAction: secondaryAction
                    )
                }
            }
        } else if permission == .denied {
            QRScannerMessageView(
                systemImageName: "video.slash",
                title: "qrScanner.permissionDenied.title".localizedString,
                message: "qrScanner.permissionDenied.body".localizedString,
                primaryAction: QRScanAction(label: "qrScanner.openSettings".localizedString) {
                    openSettings()
                },
                secondaryAction: secondaryAction
            )
        } else {
            QRScannerMessageView(
                systemImageName: "camera",
                title: "qrScanner.permission.title".localizedString,
                message: "qrScanner.permission.body".localizedString,
                primaryAction: QRScanAction(label: "qrScanner.permission.allow".localizedString) {
                    Task { permission = await CameraPermissionState.request() }
                },
                secondaryAction: secondaryAction
            )
        }
    }

    var topBar: some View {
        HStack {
            QRScannerBarButton(
                systemImageName: "xmark",
                accessibilityLabel: "qrScanner.close".localizedString,
                action: close
            )
            // A code being checked may change the account, so its result must be seen.
            .disabled(viewModel.isValidating)
            Spacer()
            if isCameraActive, !isPaused, QRTorch.isAvailable {
                QRScannerBarButton(
                    systemImageName: isTorchOn ? "flashlight.on.fill" : "flashlight.off.fill",
                    accessibilityLabel: isTorchOn
                        ? "qrScanner.torchOff".localizedString
                        : "qrScanner.torchOn".localizedString,
                    action: { setTorch(on: !isTorchOn) }
                )
            }
        }
        .padding(.horizontal, NymSpacing.large)
        .padding(.top, NymSpacing.small)
    }

    func close() {
        guard !viewModel.isValidating else { return }
        onClose()
    }

    func requestPermissionIfNeeded() async {
        guard permission == .notDetermined, !didRequestPermission else { return }
        didRequestPermission = true
        permission = await CameraPermissionState.request()
    }

    func setTorch(on: Bool) {
        guard isTorchOn != on else { return }
        isTorchOn = on
        QRTorch.set(enabled: on)
    }

    func openSettings() {
        guard let url = URL(string: UIApplication.openSettingsURLString) else { return }
        openURL(url)
    }
}

private struct QRScannerBarButton: View {
    let systemImageName: String
    let accessibilityLabel: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Image(systemName: systemImageName)
                .font(.system(size: 18, weight: .semibold))
                .foregroundStyle(Color.white)
                .frame(width: 44, height: 44)
                .background(Color.black.opacity(0.4), in: Circle())
                .contentShape(Circle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(accessibilityLabel)
    }
}
#endif
