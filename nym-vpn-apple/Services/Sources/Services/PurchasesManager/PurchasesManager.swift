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
    private enum ProductId {
        static let monthly = "1_month_may_2025"
        static let yearly = "1_year_may_2025"
    }

    private let productIds = [ProductId.monthly, ProductId.yearly]
    private var productsLoaded = false
    private var updates: Task<Void, Never>?

    @Published public var products: [Product] = []
    @Published public var isEligibleForIntroOffer: [String] = []
    @Published public var isAutoRenewEnabled = false

    /// Yearly plan price divided by 12. Currency comes from the App Store storefront — it is what Apple
    /// charges and cannot be swapped — while number formatting follows the device locale.
    public var yearlyPlanMonthlyPriceText: String? {
        guard let yearlyProduct else { return nil }
        return (yearlyProduct.price / 12).formatted(yearlyProduct.priceFormatStyle.locale(.autoupdatingCurrent))
    }

    /// Localized percentage saved by the yearly plan compared to paying monthly for a year.
    public var yearlyPlanSavingsText: String? {
        guard let yearlyProduct, let monthlyProduct else { return nil }
        let yearOfMonthlyPayments = monthlyProduct.price * 12
        guard yearOfMonthlyPayments > 0, yearlyProduct.price < yearOfMonthlyPayments else { return nil }

        let saved = (yearOfMonthlyPayments - yearlyProduct.price) / yearOfMonthlyPayments
        let rounded = (NSDecimalNumber(decimal: saved).doubleValue * 100).rounded() / 100
        return rounded.formatted(.percent.precision(.fractionLength(0)))
    }

    /// Localized free trial duration of the yearly plan, when the customer is still eligible for it.
    public var yearlyPlanFreeTrialPeriodText: String? {
        guard let yearlyProduct,
              isEligibleForIntroOffer.contains(yearlyProduct.id),
              let offer = yearlyProduct.subscription?.introductoryOffer,
              offer.price == 0
        else {
            return nil
        }
        return offer.period.formatted(yearlyProduct.subscriptionPeriodFormatStyle)
    }

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
#if SANTA
        try? await AppStore.sync()
#endif
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
    var yearlyProduct: Product? {
        products.first { $0.id == ProductId.yearly }
    }

    var monthlyProduct: Product? {
        products.first { $0.id == ProductId.monthly }
    }

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
