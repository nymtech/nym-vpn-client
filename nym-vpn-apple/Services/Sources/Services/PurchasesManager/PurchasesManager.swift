import StoreKit
import SwiftUI
import Logging
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
        static let sixMonths = "6_months_2026"
        static let yearly = "1_year_may_2025"
    }

    private let logger = Logger(label: "PurchasesManager")
    private let productIds = [ProductId.monthly, ProductId.sixMonths, ProductId.yearly]
    private let trialEligibilityProvider: NymTrialEligibilityProviding?
    private var productsLoaded = false
    private var updates: Task<Void, Never>?
    private var nymTrialEligibilityGeneration = 0
    private var introOfferEligibilityGeneration = 0

    @Published public var products: [Product] = []
    /// StoreKit's verdict only. Trial labels must go through `freeTrialVisibility(for:)`.
    @Published public var isEligibleForIntroOffer: [String] = []
    /// While true, `isEligibleForIntroOffer` may be stale: the Apple ID can have used up its trial since.
    @Published public private(set) var isRefreshingIntroOfferEligibility = false
    @Published public var isAutoRenewEnabled = false
    /// The Apple ID's live NymVPN subscription, whichever NymVPN account it pays for. Nil when there is none.
    @Published public private(set) var appleIdEntitlement: AppleIdEntitlement?
    /// The Nym account's verdict, read when the plan screen appears. Never persisted.
    @Published public private(set) var nymTrialEligibility: NymTrialEligibility = .unknown

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

    /// Localized free trial duration of the yearly plan, unless its trial label is hidden.
    /// Check `yearlyPlanFreeTrialVisibility`: while it is `.reserved` the text only keeps the layout stable.
    public var yearlyPlanFreeTrialPeriodText: String? {
        guard let yearlyProduct,
              freeTrialVisibility(for: yearlyProduct.id) != .hidden,
              let offer = yearlyProduct.subscription?.introductoryOffer,
              offer.price == 0
        else {
            return nil
        }
        return offer.period.formatted(yearlyProduct.subscriptionPeriodFormatStyle)
    }

    public var yearlyPlanFreeTrialVisibility: FreeTrialLabelVisibility {
        guard let yearlyProduct else { return .hidden }
        return freeTrialVisibility(for: yearlyProduct.id)
    }

    /// For copy that is not tied to one plan, such as the free trial benefit.
    public var anyPlanFreeTrialVisibility: FreeTrialLabelVisibility {
        FreeTrialLabelPolicy.visibility(
            isStoreKitEligible: !isEligibleForIntroOffer.isEmpty,
            isStoreKitRefreshing: isRefreshingIntroOfferEligibility,
            nymEligibility: nymTrialEligibility
        )
    }

    public init(trialEligibilityProvider: NymTrialEligibilityProviding? = nil) {
        self.trialEligibilityProvider = trialEligibilityProvider
        setup()
    }
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
            await refreshIntroOfferEligibility()
        } catch {
            print(error)
            throw error
        }
    }

    public func freeTrialVisibility(for productId: String) -> FreeTrialLabelVisibility {
        FreeTrialLabelPolicy.visibility(
            isStoreKitEligible: isEligibleForIntroOffer.contains(productId),
            isStoreKitRefreshing: isRefreshingIntroOfferEligibility,
            nymEligibility: nymTrialEligibility
        )
    }

    /// Call when a plan screen appears: StoreKit's verdict follows the Apple ID, which may have used up
    /// its trial under another Nym account since the last read.
    public func refreshTrialEligibility() async {
        async let introOfferRefresh: Void = refreshIntroOfferEligibility()
        await refreshNymTrialEligibility()
        await introOfferRefresh
    }

    public func refreshNymTrialEligibility() async {
        nymTrialEligibilityGeneration += 1
        let generation = nymTrialEligibilityGeneration
        nymTrialEligibility = .unknown

        let isEligible = await trialEligibilityProvider?.isNymTrialEligible() ?? false
        // A newer refresh replaced this one; don't overwrite its answer.
        guard generation == nymTrialEligibilityGeneration else { return }
        nymTrialEligibility = isEligible ? .eligible : .notEligible
    }

    public func purchase(with product: Product, token: String) async throws -> PurchaseOutcome {
        guard let accountToken = UUID(uuidString: token) else { return .failed }
        // Asked right before the purchase and never cached: the token has no `exp`, Apple checks its age from `iat`.
        let decision = await introOfferEligibilityDecision(for: product)
        logger.debug("Intro offer eligibility: \(decision.logDescription)")
        let options = IntroOfferEligibilityResolver.purchaseOptions(accountToken: accountToken, decision: decision)
        let result = try await product.purchase(options: options)

        let outcome: PurchaseOutcome
        switch result {
        case let .success(.verified(transaction)):
            await transaction.finish()
            outcome = .success
        case .success(.unverified):
            outcome = .failed
        case .pending:
            outcome = .pending
        case .userCancelled:
            return .userCancelled
        @unknown default:
            outcome = .failed
        }
        // Apple may have used up the trial or changed the renewal; the outcome does not wait for the re-check.
        Task {
            await refreshIntroOfferEligibility()
            await updateAutoRenewStatus()
        }
        return outcome
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
        introOfferEligibilityGeneration += 1
        isRefreshingIntroOfferEligibility = false
        nymTrialEligibility = .unknown
#if SANTA
        try? await AppStore.sync()
#endif
        try? await loadProducts()
        await updateAutoRenewStatus()
        await refreshNymTrialEligibility()
    }

    public func updateAutoRenewStatus() async {
        var renewalInfo: Product.SubscriptionInfo.RenewalInfo?
        for product in products {
            guard let subscription = product.subscription,
                  let statuses = try? await subscription.status,
                  let latestStatus = statuses.first,
                  case let .verified(info) = latestStatus.renewalInfo
            else {
                continue
            }
            renewalInfo = info
            break
        }
        isAutoRenewEnabled = renewalInfo?.willAutoRenew ?? false
        appleIdEntitlement = await currentAppleIdEntitlement(renewalInfo: renewalInfo)
        if let entitlement = appleIdEntitlement {
            // Token presence only: whether Apple sets the chooser's token on renewal info is an open question.
            let hasToken = entitlement.appAccountToken != nil
            let hasRenewalToken = entitlement.renewalAppAccountToken != nil
            let sameAccount = entitlement.appAccountToken == entitlement.renewalAppAccountToken
            logger.debug(
                "Apple ID entitlement \(entitlement.productId): hasToken=\(hasToken) hasRenewalToken=\(hasRenewalToken) sameAccount=\(sameAccount) willAutoRenew=\(entitlement.willAutoRenew)"
            )
        }
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
                await self?.refreshIntroOfferEligibility()
            }
        }
    }

    /// Fail-open: any failure or timeout purchases without the option and lets Apple's own rule decide.
    func introOfferEligibilityDecision(for product: Product) async -> IntroOfferEligibilityDecision {
        guard let provider = trialEligibilityProvider else { return .noToken(.lookupsUnavailable) }
        let productId = product.id

        return await IntroOfferEligibilityResolver.withTimeout(
            IntroOfferEligibilityResolver.timeout,
            fallback: .noToken(.timedOut)
        ) { @MainActor in
            do {
                return try await provider.withIntroOfferEligibilityLookups { lookups in
                    await IntroOfferEligibilityResolver.decide(
                        productId: productId,
                        lookups: lookups,
                        appTransactionID: PurchasesManager.verifiedAppTransactionID
                    )
                }
            } catch {
                return .noToken(.lookupsUnavailable)
            }
        }
    }

    nonisolated static func verifiedAppTransactionID() async throws -> String? {
        guard case let .verified(appTransaction) = try await AppTransaction.shared else { return nil }
        return appTransaction.appTransactionID
    }

    /// The longest-lived verified NymVPN entitlement; `currentEntitlements` already drops expired and revoked ones.
    func currentAppleIdEntitlement(
        renewalInfo: Product.SubscriptionInfo.RenewalInfo?
    ) async -> AppleIdEntitlement? {
        var latest: AppleIdEntitlement?
        for await result in Transaction.currentEntitlements {
            guard case let .verified(transaction) = result,
                  productIds.contains(transaction.productID),
                  let expirationDate = transaction.expirationDate,
                  expirationDate > (latest?.expirationDate ?? .distantPast)
            else {
                continue
            }
            latest = AppleIdEntitlement(
                productId: transaction.productID,
                expirationDate: expirationDate,
                appAccountToken: transaction.appAccountToken,
                willAutoRenew: renewalInfo?.willAutoRenew ?? false,
                renewalDate: renewalInfo?.renewalDate,
                renewalAppAccountToken: renewalInfo?.appAccountToken
            )
        }
        return latest
    }

    func refreshIntroOfferEligibility() async {
        introOfferEligibilityGeneration += 1
        let generation = introOfferEligibilityGeneration
        isRefreshingIntroOfferEligibility = true

        var eligible: [String] = []
        for plan in products {
            guard let subscription = plan.subscription else { continue }

            if await subscription.isEligibleForIntroOffer {
                eligible.append(plan.id)
            }
        }

        // A newer refresh replaced this one; don't overwrite its answer.
        guard generation == introOfferEligibilityGeneration else { return }
        isEligibleForIntroOffer = eligible
        isRefreshingIntroOfferEligibility = false
    }
}
