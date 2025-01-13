package net.nymtech.vpn.service.network

import kotlinx.coroutines.flow.Flow

internal interface NetworkService {
	val networkStatus: Flow<NetworkStatus>
}
