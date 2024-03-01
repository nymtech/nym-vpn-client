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

