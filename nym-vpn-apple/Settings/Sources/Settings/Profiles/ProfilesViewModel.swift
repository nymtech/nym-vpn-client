import Combine
import SwiftUI
import ConnectionManager
import ConnectionTypes

@MainActor public final class ProfilesViewModel: ObservableObject {
    private let connectionManager: ConnectionManager
    private var cancellables = Set<AnyCancellable>()

    @Binding private var path: NavigationPath

    let profiles = ConnectionProfile.allCases
    @Published var currentProfile: ConnectionProfile?

    public init(
        path: Binding<NavigationPath>,
        connectionManager: ConnectionManager
    ) {
        _path = path
        self.connectionManager = connectionManager
        self.currentProfile = connectionManager.currentProfile
        observe()
    }
}

extension ProfilesViewModel {
    func select(_ profile: ConnectionProfile) {
        guard profile != currentProfile else { return }
        connectionManager.setProfile(profile)
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}

private extension ProfilesViewModel {
    func observe() {
        connectionManager.$connectionConfig
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                guard let self else { return }
                currentProfile = connectionManager.currentProfile
            }
            .store(in: &cancellables)
    }
}
