package net.nymtech.nymvpn.module

import android.content.Context
import android.os.Build
import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob
import net.nymtech.logcatutil.LogCollect
import net.nymtech.logcatutil.LogcatHelper
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.service.country.CountryCacheService
import net.nymtech.nymvpn.service.country.CountryDataStoreCacheService
import net.nymtech.nymvpn.service.gateway.GatewayApi
import net.nymtech.nymvpn.service.gateway.GatewayApiService
import net.nymtech.nymvpn.service.gateway.GatewayLibService
import net.nymtech.nymvpn.service.gateway.GatewayService
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.FileUtils
import net.nymtech.vpn.Backend
import net.nymtech.vpn.NymApi
import net.nymtech.vpn.NymBackend
import nym_vpn_lib.UserAgent
import okhttp3.OkHttpClient
import retrofit2.Retrofit
import retrofit2.converter.moshi.MoshiConverterFactory
import java.util.concurrent.TimeUnit
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object AppModule {

	@Singleton
	@ApplicationScope
	@Provides
	fun providesApplicationScope(@DefaultDispatcher defaultDispatcher: CoroutineDispatcher): CoroutineScope =
		CoroutineScope(SupervisorJob() + defaultDispatcher)

	@Singleton
	@Provides
	fun provideNymApi(@IoDispatcher dispatcher: CoroutineDispatcher): NymApi {
		return NymApi(
			NymVpn.environment,
			dispatcher,
			UserAgent(BuildConfig.APP_NAME, BuildConfig.VERSION_NAME, "android-sdk${Build.VERSION.SDK_INT}", BuildConfig.COMMIT_HASH),
		)
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
	fun provideOkHttp(): OkHttpClient {
		return OkHttpClient.Builder()
			.connectTimeout(10, TimeUnit.SECONDS)
			.readTimeout(10, TimeUnit.SECONDS)
			.writeTimeout(20, TimeUnit.SECONDS)
			.build()
	}

	@Singleton
	@Provides
	fun provideRetrofit(moshi: Moshi, okHttpClient: OkHttpClient): Retrofit {
		return Retrofit.Builder()
			.client(okHttpClient)
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
	fun provideBackend(@ApplicationContext context: Context): Backend {
		return NymBackend.getInstance(context)
	}

	@Singleton
	@Provides
	fun provideLogcatHelper(@ApplicationContext context: Context): LogCollect {
		return LogcatHelper.init(context = context)
	}

	@Singleton
	@Provides
	fun provideFileUtils(@ApplicationContext context: Context, @IoDispatcher dispatcher: CoroutineDispatcher): FileUtils {
		return FileUtils(context, dispatcher)
	}
}
