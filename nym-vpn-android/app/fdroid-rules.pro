-dontwarn com.google.errorprone.annotations.**

-keepclassmembers class * extends androidx.datastore.preferences.protobuf.GeneratedMessageLite {
    <fields>;
}

# NymVPN lib
# where it's better defined where the FFI/JNI boundaries are.
-keep class net.nymtech.vpn.** { *; }
-keep class android.os.Parcelable { *; }
-keep class java.lang.Boolean { *; }
-keep class java.lang.Integer { *; }
-keep class java.lang.String { *; }
-keep class java.net.InetAddress { *; }
-keep class java.net.InetSocketAddress { *; }
-keep class java.util.ArrayList { *; }

# Keep generic signature of Call, Response (R8 full mode strips signatures from non-kept items).
 -keep,allowobfuscation,allowshrinking interface retrofit2.Call
 -keep,allowobfuscation,allowshrinking class retrofit2.Response

 # With R8 full mode generic signatures are stripped for classes that are not
 # kept. Suspend functions are wrapped in continuations where the type argument
 # is used.
 -keep,allowobfuscation,allowshrinking class kotlin.coroutines.Continuation

