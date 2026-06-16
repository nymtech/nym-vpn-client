import Foundation
import Testing
@testable import ConfigurationManager

@MainActor
@Suite struct EnvironmentChangeObserversTests {
    @Test func notifyAllFiresEveryObserver() {
        let registry = EnvironmentChangeObservers()
        var a = 0
        var b = 0
        var c = 0
        registry.add { a += 1 }
        registry.add { b += 1 }
        registry.add { c += 1 }

        registry.notifyAll()

        #expect(a == 1)
        #expect(b == 1)
        #expect(c == 1)
    }

    @Test func removedObserverDoesNotFire() {
        let registry = EnvironmentChangeObservers()
        var kept = 0
        var removed = 0
        registry.add { kept += 1 }
        let id = registry.add { removed += 1 }

        registry.remove(id)
        registry.notifyAll()

        #expect(kept == 1)
        #expect(removed == 0)
    }

    @Test func addReturnsUniqueIds() {
        let registry = EnvironmentChangeObservers()
        let id1 = registry.add {}
        let id2 = registry.add {}

        #expect(id1 != id2)
    }

    @Test func notifyAllWithNoObserversIsSafe() {
        let registry = EnvironmentChangeObservers()

        registry.notifyAll()
    }

    @Test func notifyAllFiresOnEveryCall() {
        let registry = EnvironmentChangeObservers()
        var count = 0
        registry.add { count += 1 }

        registry.notifyAll()
        registry.notifyAll()

        #expect(count == 2)
    }
}
