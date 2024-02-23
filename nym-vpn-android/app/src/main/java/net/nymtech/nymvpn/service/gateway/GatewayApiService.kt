package net.nymtech.nymvpn.service.gateway

import net.nymtech.nymvpn.service.gateway.model.DescribedGateway
import retrofit2.http.GET

interface GatewayApiService {
    @GET("gateways/described")
    suspend fun getDescribedGateways() : List<DescribedGateway>
}