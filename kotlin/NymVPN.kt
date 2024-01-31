
const val nymVPNLib = "nym_vpn_lib"

class NymVPN {
    // Load the native library "libnym_vpn_lib.so"
    init {
        System.loadLibrary(nymVPNLib)
        Log.i(tag, "loaded native library $nymVPNLib")
    }

    fun init(api_url: String,
              entry_gateway: String,
              exit_router: String,
              vpn_service: Any) {
        Log.d(tag, "calling $nymVPNLib:initVPN")
        try {
            initVPN(api_url, entry_gateway, exit_router, vpn_service)
        } catch (e: Throwable) {
            Log.e(tag, "$nymVPNLib:initVPN internal error: $e")
            Sentry.captureException(e)
        }
    }
    fun run() {
        Log.d(tag, "calling $nymNativeLib:runVPN")
        try {
            runVPN()
        } catch (e: Throwable) {
            Log.e(tag, "$nymNativeLib:runVPN internal error: $e")
            Sentry.captureException(e)
        }
    }

    fun stop() {
        Log.d(tag, "calling $nymNativeLib:stopVPN")
        try {
            stopClient()
        } catch (e: Throwable) {
            Log.e(tag, "$nymNativeLib:stopVPN internal error: $e")
            Sentry.captureException(e)
        }
    }

    private external fun initVPN(
            api_url: String,
            entry_gateway: String,
            exit_router: String,
            vpn_service: Any
    )
    private external fun runVPN()
    private external fun stopVPN()
}