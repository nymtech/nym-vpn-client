import StoreKit
import SwiftUI
import AppSettings

public final class PurchasesManager: ObservableObject {
    private let productIds = ["1_month_may_2025", "1_year_may_2025"]
    private var productsLoaded = false
    private var updates: Task<Void, Never>?

    @Published public var products = [Product]()

    public init() {
        setup()
    }

    deinit {
        updates?.cancel()
    }

    public func loadProducts() async throws {
        guard !self.productsLoaded else { return }
        Task { @MainActor in
            products = try await Product.products(for: productIds)
            guard !products.isEmpty else { return }
            productsLoaded = true
        }
    }

    public func purchase(with product: Product, token: String) async throws -> Bool {
        guard let newToken = UUID(uuidString: token) else { return false }
        let result = try await product.purchase(options: [.appAccountToken(newToken)])

        switch result {
        case let .success(.verified(transaction)):
            await transaction.finish()
            return true
        case .success(.unverified), .pending, .userCancelled:
            break
        @unknown default:
            break
        }
        return false
    }

    public func restorePurchases() async throws {
        try await AppStore.sync()
    }
}

private extension PurchasesManager {
    func setup() {
        updates = observeTransactionUpdates()

        Task {
            try? await loadProducts()
        }
    }

    func observeTransactionUpdates() -> Task<Void, Never> {
        Task(priority: .background) { 
            for await _ in Transaction.updates {}
        }
    }
}
