import GRPC
import SwiftProtobuf
import Shell

extension GRPCManager {
    public func version() async throws {
        do {
            let result = try await client.info(
                Google_Protobuf_Empty(),
                callOptions: CallOptions(timeLimit: .timeout(.seconds(3)))
            )
            Task { @MainActor in
                daemonVersion = result.version
                networkName = result.nymNetwork.networkName
                logger.info("🛜 \(result.nymNetwork.networkName)")
            }
        } catch {
            Task { @MainActor in
                guard daemonVersion != "noVersion" || daemonVersion != "update" else { return }
                daemonVersion = "noVersion"
            }
            throw error
        }
    }

    public func updateErrorReportingIfNeeded(with isEnabled: Bool) async throws {
        let isSentryEnabled = try await client.isSentryEnabled(Google_Protobuf_Empty()).value
        guard isSentryEnabled != isEnabled else { return }
        if isEnabled {
            _ = try await client.enableSentry(Google_Protobuf_Empty())
        } else {
            _ = try await client.disableSentry(Google_Protobuf_Empty())
        }
    }

    public func updateNetworkStatisticsIfNeeded(with isEnabled: Bool) async throws {
        let isStatisticsEnabled = try await client.isCollectNetworkStatsEnabled(Google_Protobuf_Empty()).value
        guard isStatisticsEnabled != isEnabled else { return }
        if isEnabled {
            _ = try await client.enableCollectNetworkStats(Google_Protobuf_Empty())
        } else {
            _ = try await client.disableCollectNetworkStats(Google_Protobuf_Empty())
        }
    }
}
