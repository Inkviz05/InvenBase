# Add project specific ProGuard rules here.
# You can control the set of applied configuration files using the
# proguardFiles setting in build.gradle.
#
# For more details, see
#   http://developer.android.com/guide/developing/tools/proguard.html

# Uncomment this to preserve the line number information for
# debugging stack traces.
#-keepattributes SourceFile,LineNumberTable

# If you keep the line number information, uncomment this to
# hide the original source file name.
#-renamesourcefileattribute SourceFile

# Retrofit
-keepattributes Signature, InnerClasses, EnclosingMethod
-keepattributes RuntimeVisibleAnnotations, RuntimeVisibleParameterAnnotations
-keepclassmembers,allowshrinking,allowobfuscation interface * {
    @retrofit2.http.* <methods>;
}
-dontwarn org.codehaus.mojo.animal_sniffer.IgnoreJRERequirement
-dontwarn javax.annotation.**
-dontwarn kotlin.Unit
-dontwarn retrofit2.KotlinExtensions
-dontwarn retrofit2.KotlinExtensions$*

# Gson
-keepattributes Signature
-keepattributes *Annotation*
-dontwarn sun.misc.**
-keep class com.google.gson.** { *; }
-keep class * implements com.google.gson.TypeAdapter
-keep class * implements com.google.gson.TypeAdapterFactory
-keep class * implements com.google.gson.JsonSerializer
-keep class * implements com.google.gson.JsonDeserializer
-keepclassmembers,allowobfuscation class * {
    @com.google.gson.annotations.SerializedName <fields>;
}

# Models are instantiated and populated reflectively by Gson. Their field names are
# defined by @SerializedName, so classes may still be optimized and obfuscated.
-keep,allowoptimization,allowobfuscation class com.invenbase.app.models.** { <fields>; }

# OkHttp
-dontwarn okhttp3.**
-dontwarn okio.**
-keepnames class okhttp3.internal.publicsuffix.PublicSuffixDatabase

# HMS probes these optional Huawei/EMUI/HarmonyOS APIs only on devices where
# they exist. They are intentionally absent from the standard Android SDK used
# by R8 in CI.
-dontwarn android.telephony.HwTelephonyManager
-dontwarn com.huawei.android.**
-dontwarn com.huawei.appgallery.log.**
-dontwarn com.huawei.hianalytics.**
-dontwarn com.huawei.libcore.io.**
-dontwarn com.huawei.ohos.localability.**
-dontwarn com.huawei.system.**
