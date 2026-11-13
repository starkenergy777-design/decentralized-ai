package ai.decentralized.app

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import java.security.KeyStore
import java.security.SecureRandom
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/** Stores the node seed encrypted by an AES key managed by Android Keystore. */
internal object AndroidPeerIdentityStore {
    private const val KEYSTORE_PROVIDER = "AndroidKeyStore"
    private const val KEY_ALIAS = "decentralized-ai-peer-seed-v1"
    private const val PREFERENCES_NAME = "peer_identity_v1"
    private const val IV_KEY = "seed_iv"
    private const val CIPHERTEXT_KEY = "seed_ciphertext"
    private const val SEED_SIZE_BYTES = 32
    private const val GCM_TAG_SIZE_BITS = 128

    fun loadOrCreate(context: Context): ByteArray {
        val preferences = context.applicationContext.getSharedPreferences(
            PREFERENCES_NAME,
            Context.MODE_PRIVATE,
        )
        readStoredSeed(preferences)?.let { return it }

        val hasPartialOrUnreadableIdentity =
            preferences.contains(IV_KEY) || preferences.contains(CIPHERTEXT_KEY)
        if (hasPartialOrUnreadableIdentity) {
            check(preferences.edit().remove(IV_KEY).remove(CIPHERTEXT_KEY).commit()) {
                "Could not clear the unreadable peer identity"
            }
            deleteKeystoreKey()
        }

        val key = getOrCreateKeystoreKey()
        val seed = ByteArray(SEED_SIZE_BYTES).also(SecureRandom()::nextBytes)
        var returnedToCaller = false
        try {
            val cipher = Cipher.getInstance("AES/GCM/NoPadding")
            cipher.init(Cipher.ENCRYPT_MODE, key)
            val ciphertext = cipher.doFinal(seed)
            val encodedIv = Base64.encodeToString(cipher.iv, Base64.NO_WRAP)
            val encodedCiphertext = Base64.encodeToString(ciphertext, Base64.NO_WRAP)
            val persisted = preferences.edit()
                .putString(IV_KEY, encodedIv)
                .putString(CIPHERTEXT_KEY, encodedCiphertext)
                .commit()
            check(persisted) { "Could not persist the encrypted peer identity" }
            returnedToCaller = true
            return seed
        } finally {
            if (!returnedToCaller) seed.fill(0)
        }
    }

    private fun readStoredSeed(preferences: android.content.SharedPreferences): ByteArray? {
        return try {
            val encodedIv = preferences.getString(IV_KEY, null) ?: return null
            val encodedCiphertext = preferences.getString(CIPHERTEXT_KEY, null) ?: return null
            val iv = Base64.decode(encodedIv, Base64.NO_WRAP)
            val ciphertext = Base64.decode(encodedCiphertext, Base64.NO_WRAP)
            val key = getExistingKeystoreKey() ?: return null
            val cipher = Cipher.getInstance("AES/GCM/NoPadding")
            cipher.init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(GCM_TAG_SIZE_BITS, iv))
            val seed = cipher.doFinal(ciphertext)
            if (seed.size == SEED_SIZE_BYTES) {
                seed
            } else {
                seed.fill(0)
                null
            }
        } catch (_: Exception) {
            null
        }
    }

    private fun getExistingKeystoreKey(): SecretKey? {
        val keyStore = loadKeystore()
        return keyStore.getKey(KEY_ALIAS, null) as? SecretKey
    }

    private fun getOrCreateKeystoreKey(): SecretKey {
        val keyStore = loadKeystore()
        (keyStore.getKey(KEY_ALIAS, null) as? SecretKey)?.let { return it }
        if (keyStore.containsAlias(KEY_ALIAS)) keyStore.deleteEntry(KEY_ALIAS)

        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, KEYSTORE_PROVIDER)
        generator.init(
            KeyGenParameterSpec.Builder(
                KEY_ALIAS,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
            )
                .setKeySize(256)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setRandomizedEncryptionRequired(true)
                .build(),
        )
        return generator.generateKey()
    }

    private fun deleteKeystoreKey() {
        val keyStore = loadKeystore()
        if (keyStore.containsAlias(KEY_ALIAS)) keyStore.deleteEntry(KEY_ALIAS)
    }

    private fun loadKeystore(): KeyStore = KeyStore.getInstance(KEYSTORE_PROVIDER).apply {
        load(null)
    }
}
