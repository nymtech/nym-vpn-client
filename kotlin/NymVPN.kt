
const val nymVPNLib = "nym_vpn_lib"

class NymVPN {
    // Load the native library "libnym_vpn_lib.so"
    init {
        System.loadLibrary(nymVPNLib)
        Log.i(tag, "loaded native library $nymVPNLib")
    }

    // this is a blocking call as `runVPN` is blocking and will releases when
    // the socks5 connection has been terminated
    fun start(api_url: String,
              entry_gateway: String,
              exit_router: String,
              vpn_service: Any) {
        Log.d(tag, "calling $nymVPNLib:runVPN")
        try {
            runVPN(api_url, entry_gateway, exit_router, vpn_service)
        } catch (e: Throwable) {
            Log.e(tag, "$nymVPNLib:runVPN internal error: $e")
            Sentry.captureException(e)
        }
    }

    private external fun runVPN(
            api_url: String,
            entry_gateway: String,
            exit_router: String,
            vpn_service: Any
    )
}