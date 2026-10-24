package com.invenbase.app.utils;

import android.content.Context;
import android.content.SharedPreferences;
import android.security.keystore.KeyGenParameterSpec;
import android.security.keystore.KeyProperties;
import android.util.Base64;

import java.nio.charset.StandardCharsets;
import java.security.KeyStore;

import javax.crypto.Cipher;
import javax.crypto.KeyGenerator;
import javax.crypto.SecretKey;
import javax.crypto.spec.GCMParameterSpec;

/** Stores sensitive strings encrypted with an AES key held by Android Keystore. */
public final class SecurePreferences {
    public static final String FILE_NAME = "secure_preferences";
    private static final String KEY_ALIAS = "invenbase_secure_preferences_key";
    private static final String TRANSFORMATION = "AES/GCM/NoPadding";

    private final SharedPreferences preferences;

    public SecurePreferences(Context context) {
        preferences = context.getApplicationContext()
                .getSharedPreferences(FILE_NAME, Context.MODE_PRIVATE);
    }

    public synchronized void putString(String key, String value) {
        if (value == null) {
            remove(key);
            return;
        }
        try {
            Cipher cipher = Cipher.getInstance(TRANSFORMATION);
            cipher.init(Cipher.ENCRYPT_MODE, getOrCreateKey());
            byte[] ciphertext = cipher.doFinal(value.getBytes(StandardCharsets.UTF_8));
            String encoded = Base64.encodeToString(cipher.getIV(), Base64.NO_WRAP)
                    + "."
                    + Base64.encodeToString(ciphertext, Base64.NO_WRAP);
            preferences.edit().putString(key, encoded).commit();
        } catch (Exception error) {
            throw new IllegalStateException("Unable to encrypt sensitive preferences", error);
        }
    }

    public synchronized String getString(String key) {
        String encoded = preferences.getString(key, null);
        if (encoded == null) {
            return null;
        }
        try {
            String[] parts = encoded.split("\\.", 2);
            if (parts.length != 2) {
                throw new IllegalStateException("Invalid encrypted preference format");
            }
            byte[] iv = Base64.decode(parts[0], Base64.NO_WRAP);
            byte[] ciphertext = Base64.decode(parts[1], Base64.NO_WRAP);
            Cipher cipher = Cipher.getInstance(TRANSFORMATION);
            cipher.init(Cipher.DECRYPT_MODE, getOrCreateKey(), new GCMParameterSpec(128, iv));
            return new String(cipher.doFinal(ciphertext), StandardCharsets.UTF_8);
        } catch (Exception error) {
            // Restored or corrupted ciphertext is unusable without its original Keystore key.
            preferences.edit().remove(key).commit();
            return null;
        }
    }

    public synchronized boolean contains(String key) {
        return preferences.contains(key);
    }

    public synchronized void remove(String key) {
        preferences.edit().remove(key).commit();
    }

    private SecretKey getOrCreateKey() throws Exception {
        KeyStore keyStore = KeyStore.getInstance("AndroidKeyStore");
        keyStore.load(null);
        java.security.Key existingKey = keyStore.getKey(KEY_ALIAS, null);
        if (existingKey instanceof SecretKey) {
            return (SecretKey) existingKey;
        }

        KeyGenerator keyGenerator = KeyGenerator.getInstance(
                KeyProperties.KEY_ALGORITHM_AES,
                "AndroidKeyStore"
        );
        keyGenerator.init(new KeyGenParameterSpec.Builder(
                KEY_ALIAS,
                KeyProperties.PURPOSE_ENCRYPT | KeyProperties.PURPOSE_DECRYPT
        )
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .build());
        return keyGenerator.generateKey();
    }
}
