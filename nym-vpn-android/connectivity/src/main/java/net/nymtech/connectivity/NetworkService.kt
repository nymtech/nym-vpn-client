package net.nymtech.connectivity

import kotlinx.coroutines.flow.Flow

interface NetworkService {
	val networkStatus: Flow<NetworkStatus>
}
