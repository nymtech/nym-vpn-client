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

@Test func mapsAccountStoreBusyToRegistrationInProgressMessage() {
    let message = ProcessingAccountErrorMapper.localizedMessage(for: VpnError.AccountStoreBusy)
    #expect(message == "errorReason.registrationInProgress".localizedString)
}

@Test func logSafeDescriptionIncludesAccountStoreBusyLabel() {
    let description = ProcessingAccountErrorMapper.logSafeDescription(for: VpnError.AccountStoreBusy)
    #expect(description == "VpnError.AccountStoreBusy")
}

@Test func mapsAllPrefetchRelatedVpnErrorsToUserFacingCopy() {
    let expectedCases: [(VpnError, String)] = [
        (.AccountStoreBusy, "errorReason.registrationInProgress".localizedString),
        (
            .ZkNymAcquisitionFailure(details: "device_not_authenticated"),
            "errorReason.noDeviceStored".localizedString
        ),
        (
            .InternalError(details: "Device time is desynced"),
            "errorReason.deviceTimeOutOfSync".localizedString
        ),
    ]
    for (error, expected) in expectedCases {
        let message = ProcessingAccountErrorMapper.localizedMessage(for: error)
        #expect(!message.isEmpty)
        #expect(message == expected, "Unexpected mapping for \(error)")
    }
}
