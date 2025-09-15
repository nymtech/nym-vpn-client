import NymVPNRpc

final class RpcAccountObserver: AccountEventObserver {
    func onAccountStateChange(newState: AccountControllerState) {
        print("RpcAccountObserver: event: \(newState)")
    }

    func onClose() {
        print("RpcAccountObserver: closed!!!")
    }
}
