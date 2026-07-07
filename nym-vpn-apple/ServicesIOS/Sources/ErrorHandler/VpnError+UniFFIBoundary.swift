#if os(iOS)
import Foundation
import NymVPNLib

public enum VpnErrorUniFFIBoundary {
    /// UniFFI foreign callbacks must throw only `VpnError`; other Swift errors become `UnexpectedUniFFICallbackError` and panic the Rust worker.
    public static func vpnError(from error: Error) -> VpnError {
        if let vpnError = error as? VpnError {
            return vpnError
        }
        return VpnError.InternalError(details: String(describing: error))
    }
}
#endif
