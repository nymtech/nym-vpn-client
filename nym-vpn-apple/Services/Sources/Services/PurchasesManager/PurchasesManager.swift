import StoreKit
import SwiftUI
import AppSettings

@MainActor public final class PurchasesManager: ObservableObject {
    private let productIds = ["1_month_may_2025", "1_year_may_2025"]
    private var productsLoaded = false
    private var updates: Task<Void, Never>?

    @Published public var products: [Product] = []
    @Published public var isEligibleForIntroOffer: [String] = []

    public init() { setup() }
    deinit { updates?.cancel() }

    public func loadProducts() async throws {
        guard !productsLoaded else { return }

        do {
            let fetched: [Product] = try await Task.detached(priority: .utility) { [productIds] in
                try await Product.products(for: productIds)
            }.value

            guard !fetched.isEmpty else { return }
            products = fetched
            productsLoaded = true
            await fetchIntroOfferEligibility()
        } catch {
            print(error)
            throw error
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
            return false
        @unknown default:
            return false
        }
    }

    public func restorePurchases() async throws {
        try await AppStore.sync()
    }
}

private extension PurchasesManager {
    func setup() {
        updates = observeTransactionUpdates()
        Task { try? await loadProducts() }
    }

    func observeTransactionUpdates() -> Task<Void, Never> {
        Task(priority: .background) {
            for await _ in Transaction.updates {}
        }
    }

    func fetchIntroOfferEligibility() async {
        var eligible: [String] = []

        for plan in products {
            guard let subscription = plan.subscription else { continue }

            if await subscription.isEligibleForIntroOffer {
                eligible.append(plan.id)
            }
        }

        await MainActor.run {
            self.isEligibleForIntroOffer = eligible
        }
    }
}
