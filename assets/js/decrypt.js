// Client-side decryption orchestration for Atuin Web
// Key management + NaCl XSalsa20Poly1305 for legacy history + PASETO V4 for records
(function() {
  "use strict";

  var KEY_STORAGE = "atuin-encryption-key";

  function getKey() {
    var b64 = sessionStorage.getItem(KEY_STORAGE);
    if (!b64) return null;
    try {
      var raw = atob(b64);
      var bytes = new Uint8Array(raw.length);
      for (var i = 0; i < raw.length; i++) bytes[i] = raw.charCodeAt(i);
      return bytes;
    } catch (e) {
      return null;
    }
  }

  function storeKey(keyBytes) {
    var b64 = btoa(String.fromCharCode.apply(null, keyBytes));
    sessionStorage.setItem(KEY_STORAGE, b64);
  }

  function hasKey() {
    return sessionStorage.getItem(KEY_STORAGE) !== null;
  }

  function removeKey() {
    sessionStorage.removeItem(KEY_STORAGE);
  }

  function updateKeyStatus() {
    var btn = document.getElementById("keyStatusBtn");
    var text = document.getElementById("keyStatusText");
    if (!btn || !text) return;

    if (hasKey()) {
      btn.classList.add("has-key");
      text.textContent = "Key loaded";
    } else {
      btn.classList.remove("has-key");
      text.textContent = "No key loaded";
    }
  }

  // Decrypt legacy history entry (XSalsa20Poly1305)
  // Atuin format: nonce (24 bytes) + ciphertext, base64-encoded
  function decryptLegacyHistory(b64Data, key) {
    if (!b64Data || !key || typeof nacl === "undefined") return null;

    try {
      var raw = atob(b64Data);
      var data = new Uint8Array(raw.length);
      for (var i = 0; i < raw.length; i++) data[i] = raw.charCodeAt(i);

      // First 24 bytes are nonce, rest is ciphertext
      var nonce = data.slice(0, 24);
      var ciphertext = data.slice(24);

      var plaintext = nacl.secretbox.open(ciphertext, nonce, key);
      if (!plaintext) return null;

      // Plaintext is MessagePack-encoded
      if (typeof MessagePack !== "undefined") {
        var decoded = MessagePack.decode(plaintext);
        if (decoded && decoded.command) return decoded.command;
        if (typeof decoded === "string") return decoded;
        return JSON.stringify(decoded);
      }

      // Fallback: try UTF-8 decode
      return new TextDecoder().decode(plaintext);
    } catch (e) {
      console.warn("Decryption failed:", e);
      return null;
    }
  }

  // Process all encrypted elements on the page
  function processEncryptedElements() {
    var key = getKey();
    if (!key) return;

    var elements = document.querySelectorAll("[data-encrypted]");
    elements.forEach(function(el) {
      var data = el.getAttribute("data-encrypted");
      if (!data || data === "") return;

      // Check for PASETO V4 (records)
      var cek = el.getAttribute("data-cek");
      if (cek) {
        // Records use PASETO V4 - not yet implemented
        return;
      }

      // Legacy history decryption
      var plaintext = decryptLegacyHistory(data, key);
      if (plaintext) {
        el.innerHTML = "";
        var code = document.createElement("code");
        code.className = "font-mono";
        code.textContent = plaintext;
        el.appendChild(code);
      }
    });
  }

  // Copy a command hint to clipboard
  window.copyHintToClipboard = function(btn, text) {
    navigator.clipboard.writeText(text).then(function() {
      var original = btn.innerHTML;
      btn.innerHTML = "Copied!";
      btn.classList.add("btn-success");
      btn.classList.remove("btn-outline-secondary");
      setTimeout(function() {
        btn.innerHTML = original;
        btn.classList.remove("btn-success");
        btn.classList.add("btn-outline-secondary");
      }, 1500);
    });
  };

  // Global functions for the key modal
  window.loadEncryptionKey = async function() {
    var errorEl = document.getElementById("keyError");
    var successEl = document.getElementById("keySuccess");
    errorEl.classList.add("d-none");
    successEl.classList.add("d-none");

    // Check which tab is active
    var b64Tab = document.getElementById("keyBase64Tab");
    var isBase64 = b64Tab && b64Tab.classList.contains("show");

    try {
      var keyBytes;

      if (isBase64) {
        var b64Input = document.getElementById("keyBase64Input").value.trim();
        if (!b64Input) throw new Error("Please enter a key");

        var raw = atob(b64Input);
        keyBytes = new Uint8Array(raw.length);
        for (var i = 0; i < raw.length; i++) keyBytes[i] = raw.charCodeAt(i);
      } else {
        var mnemonic = document.getElementById("keyMnemonicInput").value.trim();
        if (!mnemonic) throw new Error("Please enter a mnemonic");

        if (typeof BIP39 === "undefined") throw new Error("BIP39 library not loaded");
        keyBytes = await BIP39.mnemonicToEntropy(mnemonic);
      }

      if (keyBytes.length !== 32) {
        throw new Error("Key must be 32 bytes, got " + keyBytes.length);
      }

      storeKey(keyBytes);
      updateKeyStatus();
      processEncryptedElements();

      successEl.classList.remove("d-none");

      // Close modal after a short delay
      setTimeout(function() {
        var modal = bootstrap.Modal.getInstance(document.getElementById("keyModal"));
        if (modal) modal.hide();
      }, 1000);
    } catch (e) {
      errorEl.textContent = e.message;
      errorEl.classList.remove("d-none");
    }
  };

  window.clearEncryptionKey = function() {
    removeKey();
    updateKeyStatus();

    // Restore encrypted badges
    document.querySelectorAll("[data-encrypted]").forEach(function(el) {
      var data = el.getAttribute("data-encrypted");
      if (data) {
        el.innerHTML = '<span class="badge badge-encrypted">[encrypted]</span>';
      }
    });

    var successEl = document.getElementById("keySuccess");
    var errorEl = document.getElementById("keyError");
    if (successEl) successEl.classList.add("d-none");
    if (errorEl) errorEl.classList.add("d-none");
  };

  // Run on page load and after htmx swaps
  function init() {
    updateKeyStatus();
    processEncryptedElements();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }

  // Re-process after htmx content swaps
  document.addEventListener("htmx:afterSwap", function() {
    processEncryptedElements();
  });
})();
