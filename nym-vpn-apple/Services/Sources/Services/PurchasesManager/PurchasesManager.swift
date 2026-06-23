import StoreKit
import SwiftUI
import AppSettings
#if SANTA
import ConfigurationManager
#endif

public enum PurchaseOutcome: Equatable, Sendable {
    case success
    case userCancelled
    case pending
    case failed
}

@MainActor public final class PurchasesManager: ObservableObject {
    private let productIds = ["1_month_may_2025", "1_year_may_2025"]
    private var productsLoaded = false
    private var updates: Task<Void, Never>?

    @Published public var products: [Product] = []
    @Published public var isEligibleForIntroOffer: [String] = []
    @Published public var isAutoRenewEnabled = false

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

    public func purchase(with product: Product, token: String) async throws -> PurchaseOutcome {
        guard let accountToken = UUID(uuidString: token) else { return .failed }
        let result = try await product.purchase(options: [.appAccountToken(accountToken)])

        switch result {
        case let .success(.verified(transaction)):
            await transaction.finish()
            return .success
        case .success(.unverified):
            return .failed
        case .pending:
            return .pending
        case .userCancelled:
            return .userCancelled
        @unknown default:
            return .failed
        }
    }

    public func restorePurchases() async throws {
        try await AppStore.sync()
    }

#if SANTA
    public func registerForEnvironmentChanges(configurationManager: ConfigurationManager) {
        configurationManager.addEnvironmentDidChangeObserver { [weak self] in
            Task { @MainActor in
                await self?.resetForEnvironmentChange()
            }
        }
    }
#endif

    public func resetForEnvironmentChange() async {
        productsLoaded = false
        products = []
        isEligibleForIntroOffer = []
        try? await loadProducts()
        await updateAutoRenewStatus()
    }

    public func updateAutoRenewStatus() async {
        var autoRenew = false
        for product in products {
            guard let subscription = product.subscription,
                  let statuses = try? await subscription.status,
                  let latestStatus = statuses.first
            else {
                continue
            }

            if case let .verified(renewalInfo) = latestStatus.renewalInfo {
                autoRenew = renewalInfo.willAutoRenew
                break
            }
        }
        isAutoRenewEnabled = autoRenew
    }
}

private extension PurchasesManager {
    func setup() {
        updates = observeTransactionUpdates()
        Task {
            try? await loadProducts()
            await updateAutoRenewStatus()
        }
    }

    func observeTransactionUpdates() -> Task<Void, Never> {
        Task(priority: .background) { [weak self] in
            for await _ in Transaction.updates {
                await self?.updateAutoRenewStatus()
            }
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
