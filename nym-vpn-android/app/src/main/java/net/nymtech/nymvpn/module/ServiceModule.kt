package net.nymtech.nymvpn.module

import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.service.country.CountryCacheService
import net.nymtech.nymvpn.service.country.CountryDataStoreCacheService
import net.nymtech.nymvpn.service.gateway.GatewayApi
import net.nymtech.nymvpn.service.gateway.GatewayApiService
import net.nymtech.nymvpn.service.gateway.GatewayLibService
import net.nymtech.nymvpn.service.gateway.GatewayService
import net.nymtech.nymvpn.util.Constants
import net.nymtech.vpn.NymApi
import net.nymtech.vpn.NymVpnClient
import net.nymtech.vpn.VpnClient
import retrofit2.Retrofit
import retrofit2.converter.moshi.MoshiConverterFactory
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
class ServiceModule {

	@Singleton
	@Provides
	fun provideNymApi(): NymApi {
		return NymApi(NymVpn.environment)
	}

	@Singleton
	@Provides
	fun provideMoshi(): Moshi {
		return Moshi.Builder()
			.add(KotlinJsonAdapterFactory())
			.build()
	}

	@Singleton
	@Provides
	fun provideGatewayService(retrofit: Retrofit): GatewayApi {
		return retrofit.create(GatewayApi::class.java)
	}

	@Singleton
	@Provides
	fun provideRetrofit(moshi: Moshi): Retrofit {
		return Retrofit.Builder()
			.addConverterFactory(MoshiConverterFactory.create(moshi))
			.baseUrl(Constants.VPN_API_BASE_URL)
			.build()
	}

	@Native
	@Singleton
	@Provides
	fun provideGatewayLibService(nymApi: NymApi): GatewayService {
		return GatewayLibService(nymApi)
	}

	@Android
	@Singleton
	@Provides
	fun provideGatewayApiService(gatewayApi: GatewayApi, gatewayLibService: GatewayLibService): GatewayService {
		return GatewayApiService(gatewayApi, gatewayLibService)
	}

	@Singleton
	@Provides
	fun provideCountryCacheService(@Android gatewayService: GatewayService, gatewayRepository: GatewayRepository): CountryCacheService {
		return CountryDataStoreCacheService(gatewayRepository, gatewayService)
	}

	@Singleton
	@Provides
	fun provideVpnClient(): VpnClient {
		return NymVpnClient.init(environment = NymVpn.environment)
	}
}
