package net.nymtech.vpn

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class NymVpnClientTest {
    @Test
    fun useAppContext() {
        //TODO write tests
        // Context of the app under test.
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        //assertEquals("net.nymtech.vpn_client.test", appContext.packageName)
        //NymVpnClient.connect(context, EntryPoint.Location("DE"), )
    }
}