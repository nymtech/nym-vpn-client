import Foundation

@Observable
@MainActor
public final class SnackbarManager {
    public static let shared = SnackbarManager()

    public private(set) var current: SnackbarItem?
    @ObservationIgnored private var queue: [SnackbarItem] = []
    @ObservationIgnored private var dismissTask: Task<Void, Never>?

    public init() {}

    public func enqueue(_ item: SnackbarItem) {
        if current == nil {
            present(item)
        } else {
            queue.append(item)
        }
    }

    public func dismiss() {
        dismissTask?.cancel()
        dismissTask = nil
        if queue.isEmpty {
            current = nil
        } else {
            present(queue.removeFirst())
        }
    }

    public func clear() {
        dismissTask?.cancel()
        dismissTask = nil
        queue.removeAll()
        current = nil
    }
}

private extension SnackbarManager {
    func present(_ item: SnackbarItem) {
        current = item
        dismissTask?.cancel()
        guard let duration = item.duration else {
            dismissTask = nil
            return
        }
        let id = item.id
        dismissTask = Task { [weak self] in
            try? await Task.sleep(for: .seconds(duration))
            guard !Task.isCancelled else { return }
            await MainActor.run {
                guard let self, self.current?.id == id else { return }
                self.dismiss()
            }
        }
    }
}
