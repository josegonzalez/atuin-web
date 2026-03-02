// Client-side decryption orchestration for Atuin Web
// Key management + PASETO V4 for records
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

  // Tag-specific record decoders
  function decodeRecord(tag, innerBytes) {
    // Unwrap outer: version + bin/data blob
    var outer = [];
    for (var ov of MessagePack.decodeMulti(innerBytes)) outer.push(ov);
    var blob = outer.length > 1 ? outer[1] : outer[0];

    switch (tag) {
      case "kv":
        return decodeKv(blob);
      case "config-shell-alias":
        return decodeAlias(blob);
      case "dotfiles-var":
        return decodeVar(blob);
      case "script":
        return decodeScript(blob);
      default:
        return decodeHistory(blob);
    }
  }

  function decodeHistory(blob) {
    var decoded;
    if (blob instanceof Uint8Array) {
      var inner = [];
      for (var iv of MessagePack.decodeMulti(blob)) inner.push(iv);
      decoded = inner.length > 1 ? inner[inner.length - 1] : inner[0];
    } else {
      decoded = blob;
    }

    if (decoded && decoded.command) {
      return decoded.command;
    } else if (Array.isArray(decoded)) {
      // rmp_serde compact format: struct as array
      // History: [id, timestamp, duration, exit, command, cwd, session, hostname, deleted_at]
      return decoded[4] || JSON.stringify(decoded);
    }
    return JSON.stringify(decoded);
  }

  function decodeKv(blob) {
    // KV: flat array [namespace, key, has_value, value?]
    var decoded;
    if (blob instanceof Uint8Array) {
      decoded = MessagePack.decode(blob);
    } else {
      decoded = blob;
    }

    if (Array.isArray(decoded)) {
      var ns = decoded[0] || "";
      var key = decoded[1] || "";
      var hasValue = decoded[2];
      if (hasValue && decoded.length > 3) {
        var val = decoded[3];
        if (val instanceof Uint8Array) {
          val = new TextDecoder().decode(val);
        }
        return ns + ":" + key + " = " + val;
      }
      return ns + ":" + key + " (deleted)";
    }
    return JSON.stringify(decoded);
  }

  function decodeAlias(blob) {
    // Alias: u8(discriminant) + Create: array(2)[name, value], Delete: array(1)[name]
    var parts = [];
    if (blob instanceof Uint8Array) {
      for (var v of MessagePack.decodeMulti(blob)) parts.push(v);
    } else {
      parts = [blob];
    }

    var discriminant = (parts.length > 1) ? parts[0] : 0;
    var data = parts.length > 1 ? parts[1] : parts[0];

    if (discriminant === 0 && Array.isArray(data) && data.length >= 2) {
      return "alias " + data[0] + "='" + data[1] + "'";
    } else if (discriminant === 1 && Array.isArray(data) && data.length >= 1) {
      return "unalias " + data[0];
    }
    return JSON.stringify(data);
  }

  function decodeVar(blob) {
    // Var: u8(discriminant) + Create: array(3)[name, value, export], Delete: array(1)[name]
    var parts = [];
    if (blob instanceof Uint8Array) {
      for (var v of MessagePack.decodeMulti(blob)) parts.push(v);
    } else {
      parts = [blob];
    }

    var discriminant = (parts.length > 1) ? parts[0] : 0;
    var data = parts.length > 1 ? parts[1] : parts[0];

    if (discriminant === 0 && Array.isArray(data) && data.length >= 3) {
      var name = data[0];
      var value = data[1];
      var isExport = data[2];
      if (isExport) {
        return "export " + name + "=" + value;
      }
      return name + "=" + value;
    } else if (discriminant === 1 && Array.isArray(data) && data.length >= 1) {
      return "unset " + data[0];
    }
    return JSON.stringify(data);
  }

  function decodeScript(blob) {
    // Script: u8(discriminant) + Create/Update: bin(array(6)[id,name,desc,shebang,tags,script]),
    //                            Delete: str(uuid)
    var parts = [];
    if (blob instanceof Uint8Array) {
      for (var v of MessagePack.decodeMulti(blob)) parts.push(v);
    } else {
      parts = [blob];
    }

    var discriminant = (parts.length > 1) ? parts[0] : 0;
    var data = parts.length > 1 ? parts[1] : parts[0];

    if ((discriminant === 0 || discriminant === 2) && data instanceof Uint8Array) {
      // Inner bin contains msgpack array
      var scriptData = MessagePack.decode(data);
      if (Array.isArray(scriptData) && scriptData.length >= 3) {
        var name = scriptData[1] || "";
        var desc = scriptData[2] || "";
        return name + (desc ? " \u2014 " + desc : "");
      }
      return JSON.stringify(scriptData);
    } else if (discriminant === 1 && typeof data === "string") {
      return "delete " + data;
    }
    return JSON.stringify(data);
  }

  // Expose decoders for testing
  window.AtuinDecoders = {
    decodeRecord: decodeRecord,
    decodeHistory: decodeHistory,
    decodeKv: decodeKv,
    decodeAlias: decodeAlias,
    decodeVar: decodeVar,
    decodeScript: decodeScript
  };

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

  // Process all encrypted elements on the page
  function processEncryptedElements() {
    var key = getKey();
    if (!key) return;

    var elements = document.querySelectorAll("[data-encrypted]");
    elements.forEach(function(el) {
      var data = el.getAttribute("data-encrypted");
      if (!data || data === "") return;

      // Check for PASETO V4 (records)
      var cekAttr = el.getAttribute("data-cek");
      if (cekAttr) {
        try {
          if (typeof PasetoV4 === "undefined" || typeof blake2b === "undefined") return;

          // Parse CEK wrapper — could be JSON {"wpk":"..."} or direct PASERK string
          var wpk = cekAttr;
          if (cekAttr.charAt(0) === "{") {
            var cekData = JSON.parse(cekAttr);
            wpk = cekData.wpk;
          }
          if (!wpk) return;

          // Unwrap CEK using master key
          var cekBytes = PasetoV4.unwrapPIE(wpk, key);
          if (!cekBytes) return;

          // Build implicit assertion from record metadata
          var implicit = JSON.stringify({
            id: el.getAttribute("data-rid") || "",
            idx: parseInt(el.getAttribute("data-ridx") || "0", 10),
            version: el.getAttribute("data-rversion") || "",
            tag: el.getAttribute("data-rtag") || "",
            host: el.getAttribute("data-rhost") || ""
          });

          // Decrypt PASETO token (try with implicit assertion, fall back to without)
          var plaintext = PasetoV4.decrypt(data, cekBytes, implicit);
          if (!plaintext) {
            plaintext = PasetoV4.decrypt(data, cekBytes, "");
          }
          if (!plaintext) return;

          // Parse AtuinPayload: {"data":"<base64url>"}
          var payload = JSON.parse(plaintext);
          var innerBytes = PasetoV4._base64urlDecode(payload.data);

          // Atuin record data: nested msgpack with version prefixes.
          // Outer: msgpack(version) + msgpack(bin/data)
          // Use decodeMulti at each level since decode() throws on extra bytes.
          var rtag = el.getAttribute("data-rtag") || "";
          var display;
          if (typeof MessagePack !== "undefined") {
            display = decodeRecord(rtag, innerBytes);
          } else {
            display = new TextDecoder().decode(innerBytes);
          }

          el.innerHTML = "";
          var wrapper = document.createElement("span");
          wrapper.className = "copyable-command";
          wrapper.title = "Click to copy";
          wrapper.style.cursor = "pointer";
          var code = document.createElement("code");
          code.className = "font-mono";
          code.textContent = display;
          wrapper.appendChild(code);
          wrapper.addEventListener("click", function() {
            navigator.clipboard.writeText(display).then(function() {
              var toastEl = document.getElementById("copyToast");
              if (toastEl) {
                var toast = bootstrap.Toast.getOrCreateInstance(toastEl);
                toast.show();
              }
            });
          });
          el.appendChild(wrapper);
        } catch (e) {
          console.warn("Record decryption failed:", e);
        }
        return;
      }

    });
  }

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

      // Atuin stores the key as msgpack-encoded array, then base64.
      // If we get != 32 bytes, try msgpack decoding to unwrap.
      if (keyBytes.length !== 32 && typeof MessagePack !== "undefined") {
        try {
          var decoded = MessagePack.decode(keyBytes);
          if (Array.isArray(decoded) && decoded.length === 32) {
            keyBytes = new Uint8Array(decoded);
          }
        } catch (e) { /* not msgpack, fall through to size check */ }
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
