import Foundation
import AccountPrefetchGates

// All five requirements are already satisfied by CredentialsManager's existing
// @MainActor methods (ensureCredentialImportResolved, prepareRegisteredAccount,
// updateAccountSummary(force:untilActive:), isAccountActive, prefetchZkNyms(timeout:)),
// so the conformance is declaration-only. Cross-platform: the iOS-only work lives
// behind the methods' own `#if os(iOS)` bodies with macOS fallbacks.
extension CredentialsManager: AccountProcessing {}
