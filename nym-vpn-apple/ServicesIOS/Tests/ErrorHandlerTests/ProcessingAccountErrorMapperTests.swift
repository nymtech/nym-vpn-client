import Testing
@testable import ErrorHandler
import NymVPNLib

@Test func mapsMaxDevicesInternalError() {
    let message = ProcessingAccountErrorMapper.localizedMessage(
        for: VpnError.InternalError(details: "Maximum number of devices reached")
    )
    #expect(message == "errorReason.maxDevicesReached".localizedString)
}

@Test func mapsFairUsageDepletedInternalError() {
    let message = ProcessingAccountErrorMapper.localizedMessage(
        for: VpnError.InternalError(details: "Fair usage depleted")
    )
    #expect(message == "errorReason.bandwidthExceeded".localizedString)
}

@Test func mapsDeviceTimeDesyncedInternalError() {
    let message = ProcessingAccountErrorMapper.localizedMessage(
        for: VpnError.InternalError(details: "Device time is desynced")
    )
    #expect(message == "errorReason.deviceTimeOutOfSync".localizedString)
}

@Test func logSafeDescriptionIncludesVpnErrorDetails() {
    let description = ProcessingAccountErrorMapper.logSafeDescription(
        for: VpnError.ZkNymAcquisitionFailure(details: "device_not_authenticated")
    )
    #expect(description.contains("device_not_authenticated"))
}

@Test func mapsZkNymDeviceNotAuthenticatedFailure() {
    let message = ProcessingAccountErrorMapper.localizedMessage(
        for: VpnError.ZkNymAcquisitionFailure(
            details: "device_not_authenticated from API"
        )
    )
    #expect(message == "errorReason.noDeviceStored".localizedString)
}
