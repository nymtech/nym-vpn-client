import Foundation

final class EnvironmentChangeObservers {
    private var observers: [UUID: () -> Void] = [:]

    @discardableResult
    func add(_ handler: @escaping () -> Void) -> UUID {
        let id = UUID()
        observers[id] = handler
        return id
    }

    func remove(_ id: UUID) {
        observers.removeValue(forKey: id)
    }

    func notifyAll() {
        observers.values.forEach { $0() }
    }
}
