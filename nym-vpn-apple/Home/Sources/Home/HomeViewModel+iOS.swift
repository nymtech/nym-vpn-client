#if os(iOS)
extension HomeViewModel {
    func updateTimeConnected() {
        Task { @MainActor in
            guard let activeTunnel,
                  activeTunnel.status == .connected,
                  let connectedDate = activeTunnel.tunnel.connection.connectedDate
            else {
                timeConnected = nil
                return
            }
            timeConnected = connectedDate
        }
    }
}
#endif
