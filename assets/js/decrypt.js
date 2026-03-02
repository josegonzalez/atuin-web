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
    try {
      for (var ov of MessagePack.decodeMulti(innerBytes)) outer.push(ov);
    } catch (e) {
      // Malformed msgpack — e.g. fixarray header declares more elements
      // than present (deleted KV with omitted None value).
      // Try extracting individual elements after the array header.
      var fb = innerBytes[0];
      if (fb >= 0x90 && fb <= 0x9f) {
        var elems = [];
        try {
          for (var v of MessagePack.decodeMulti(innerBytes.slice(1))) elems.push(v);
        } catch (_) { /* use what we got */ }
        if (elems.length > 0) outer = [elems];
      }
      if (outer.length === 0) throw e;
    }
    // Strip leading version string if present
    // (History/KV may include version prefix; alias/var/script data has none)
    var elements = outer;
    if (elements.length > 0 && typeof elements[0] === "string") {
      elements = elements.slice(1);
    }

    // Single remaining element → unwrap; multiple → pass as array
    var blob = elements.length === 1 ? elements[0] : elements;

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
    var parts = [];
    if (Array.isArray(blob)) {
      parts = blob;
    } else if (blob instanceof Uint8Array) {
      for (var v of MessagePack.decodeMulti(blob)) parts.push(v);
    } else {
      parts = [blob];
    }

    var discriminant = (parts.length > 1) ? parts[0] : 0;
    var data = parts.length > 1 ? parts[1] : parts[0];

    // Delete variant (discriminant 1)
    if (discriminant === 1) {
      var deleteId = (typeof data === "string") ? data : JSON.stringify(data);
      return {text: "delete " + deleteId, deleted: true};
    }

    // Create variant (discriminant 0) — data is Uint8Array containing version + struct
    var decoded;
    if (data instanceof Uint8Array) {
      var inner = [];
      for (var iv of MessagePack.decodeMulti(data)) inner.push(iv);
      decoded = inner.length > 1 ? inner[inner.length - 1] : inner[0];
    } else {
      decoded = data;
    }

    if (decoded && decoded.command) {
      return {text: decoded.command, deleted: false};
    } else if (Array.isArray(decoded)) {
      // rmp_serde compact format: struct as array
      // History: [id, timestamp, duration, exit, command, cwd, session, hostname, deleted_at]
      return {text: decoded[4] || JSON.stringify(decoded), deleted: false};
    }
    return {text: JSON.stringify(decoded), deleted: false};
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
        return {text: ns + ":" + key + " = " + val, deleted: false};
      }
      return {text: ns + ":" + key + " (deleted)", deleted: true};
    }
    return {text: JSON.stringify(decoded), deleted: false};
  }

  function decodeAlias(blob) {
    // Alias: u8(discriminant) + Create: array(2)[name, value], Delete: array(1)[name]
    var parts = [];
    if (Array.isArray(blob)) {
      // Pre-split from decodeRecord (raw outer format: 3+ elements)
      parts = blob;
    } else if (blob instanceof Uint8Array) {
      for (var v of MessagePack.decodeMulti(blob)) parts.push(v);
    } else {
      parts = [blob];
    }

    var discriminant = (parts.length > 1) ? parts[0] : 0;
    var data = parts.length > 1 ? parts[1] : parts[0];

    if (discriminant === 0 && Array.isArray(data) && data.length >= 2) {
      return {text: "alias " + data[0] + "='" + data[1] + "'", deleted: false};
    } else if (discriminant === 1 && Array.isArray(data) && data.length >= 1) {
      return {text: "unalias " + data[0], deleted: true};
    }
    return {text: JSON.stringify(data), deleted: false};
  }

  function decodeVar(blob) {
    // Var: u8(discriminant) + Create: array(3)[name, value, export], Delete: array(1)[name]
    var parts = [];
    if (Array.isArray(blob)) {
      parts = blob;
    } else if (blob instanceof Uint8Array) {
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
        return {text: "export " + name + "=" + value, deleted: false};
      }
      return {text: name + "=" + value, deleted: false};
    } else if (discriminant === 1 && Array.isArray(data) && data.length >= 1) {
      return {text: "unset " + data[0], deleted: true};
    }
    return {text: JSON.stringify(data), deleted: false};
  }

  function decodeScript(blob) {
    // Script: u8(discriminant) + Create/Update: bin(array(6)[id,name,desc,shebang,tags,script]),
    //                            Delete: str(uuid)
    var parts = [];
    if (Array.isArray(blob)) {
      parts = blob;
    } else if (blob instanceof Uint8Array) {
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
        var shebang = scriptData.length > 3 ? (scriptData[3] || "") : "";
        var tags = scriptData.length > 4 ? (scriptData[4] || []) : [];
        var scriptBody = scriptData.length > 5 ? (scriptData[5] || "") : "";
        return {
          text: name + (desc ? " \u2014 " + desc : ""),
          deleted: false,
          script: { name: name, description: desc, shebang: shebang, tags: tags, body: scriptBody }
        };
      }
      return {text: JSON.stringify(scriptData), deleted: false};
    } else if (discriminant === 1 && typeof data === "string") {
      return {text: "delete " + data, deleted: true};
    }
    return {text: JSON.stringify(data), deleted: false};
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
          var result;
          if (typeof MessagePack !== "undefined") {
            result = decodeRecord(rtag, innerBytes);
          } else {
            result = {text: new TextDecoder().decode(innerBytes), deleted: false};
          }
          var display = (typeof result === "object") ? result.text : result;
          var isDeleted = (typeof result === "object") ? result.deleted : false;

          el.innerHTML = "";
          var wrapper = document.createElement("span");
          wrapper.className = "copyable-command";
          wrapper.title = "Click to copy";
          wrapper.style.cursor = "pointer";
          var code = document.createElement("code");
          code.className = "font-mono";
          code.textContent = display;
          wrapper.appendChild(code);
          if (isDeleted) {
            var badge = document.createElement("span");
            badge.className = "badge badge-deleted ms-2";
            badge.textContent = "deleted";
            wrapper.appendChild(badge);
          }
          var copyRef = {text: display};
          var wrapperClickFn = function() {
            navigator.clipboard.writeText(copyRef.text).then(function() {
              var toastEl = document.getElementById("copyToast");
              if (toastEl) {
                var toast = bootstrap.Toast.getOrCreateInstance(toastEl);
                toast.show();
              }
            });
          };
          wrapper.addEventListener("click", function() { wrapperClickFn(); });
          el.appendChild(wrapper);

          // Expandable detail row for script records
          if (result && result.script && result.script.body) {
            var parentRow = el.closest("tr");
            if (parentRow) {
              // Clean up any existing detail row (handles htmx re-swaps)
              var nextRow = parentRow.nextElementSibling;
              if (nextRow && nextRow.hasAttribute("data-script-detail")) {
                nextRow.parentNode.removeChild(nextRow);
              }

              // Add chevron toggle before the wrapper content
              var chevron = document.createElement("span");
              chevron.className = "script-toggle";
              chevron.textContent = "\u25B6";
              chevron.title = "Show script details";
              chevron.style.cursor = "pointer";
              chevron.style.marginRight = "6px";
              chevron.style.display = "inline-block";
              chevron.style.transition = "transform 0.15s ease";
              el.insertBefore(chevron, wrapper);

              // Add tag badges after the wrapper
              var scriptTags = result.script.tags;
              if (Array.isArray(scriptTags) && scriptTags.length > 0) {
                for (var ti = 0; ti < scriptTags.length; ti++) {
                  var tagBadge = document.createElement("span");
                  tagBadge.className = "badge bg-secondary ms-1";
                  tagBadge.style.fontSize = "0.7rem";
                  tagBadge.textContent = scriptTags[ti];
                  el.appendChild(tagBadge);
                }
              }

              // Build detail sub-row
              var detailRow = document.createElement("tr");
              detailRow.setAttribute("data-script-detail", "true");
              detailRow.className = "script-detail-row";

              var detailCell = document.createElement("td");
              detailCell.setAttribute("colspan", "4");
              detailCell.style.padding = "0";
              detailCell.style.borderBottom = "1px solid var(--bs-border-color)";

              var detailContent = document.createElement("div");
              detailContent.className = "script-detail-content";
              detailContent.style.display = "none";
              detailContent.style.padding = "0.75rem 1rem";
              detailContent.style.background = "var(--surface-200)";

              // Shebang line
              if (result.script.shebang) {
                var shebangEl = document.createElement("div");
                shebangEl.style.fontSize = "0.75rem";
                shebangEl.style.color = "var(--bs-tertiary-color)";
                shebangEl.style.marginBottom = "0.5rem";
                shebangEl.textContent = result.script.shebang;
                detailContent.appendChild(shebangEl);
              }

              // Script body
              var pre = document.createElement("pre");
              pre.style.maxHeight = "300px";
              pre.style.overflow = "auto";
              pre.style.margin = "0";
              pre.style.background = "var(--surface-100)";
              pre.style.border = "1px solid var(--bs-border-color)";
              pre.style.borderRadius = "4px";
              pre.style.padding = "0.75rem";
              var bodyCode = document.createElement("code");
              bodyCode.className = "font-mono";
              bodyCode.textContent = result.script.body;
              pre.appendChild(bodyCode);
              detailContent.appendChild(pre);

              // Copy button
              var copyBtn = document.createElement("button");
              copyBtn.className = "btn btn-sm btn-outline-secondary mt-2";
              copyBtn.textContent = "Copy script";
              copyBtn.style.fontSize = "0.75rem";
              var scriptBodyText = result.script.body;
              copyBtn.addEventListener("click", function() {
                navigator.clipboard.writeText(scriptBodyText).then(function() {
                  var toastEl = document.getElementById("copyToast");
                  if (toastEl) {
                    var toast = bootstrap.Toast.getOrCreateInstance(toastEl);
                    toast.show();
                  }
                });
              });
              detailContent.appendChild(copyBtn);

              detailCell.appendChild(detailContent);
              detailRow.appendChild(detailCell);
              parentRow.parentNode.insertBefore(detailRow, parentRow.nextSibling);

              // Toggle expand/collapse
              var toggleDetail = function() {
                if (detailContent.style.display === "none") {
                  detailContent.style.display = "block";
                  chevron.textContent = "\u25BC";
                  chevron.title = "Hide script details";
                } else {
                  detailContent.style.display = "none";
                  chevron.textContent = "\u25B6";
                  chevron.title = "Show script details";
                }
              };
              chevron.addEventListener("click", function(e) {
                e.stopPropagation();
                toggleDetail();
              });

              // Override wrapper click to toggle instead of copy
              wrapperClickFn = toggleDetail;
              wrapper.title = "Click to expand";
            }
          }
        } catch (e) {
          console.warn("Record decryption/decode failed:", e);

          // Build full detail object for inspection
          var detail = {
            error: { message: e.message, type: e.name },
            record: {
              id: el.getAttribute("data-rid") || "",
              idx: el.getAttribute("data-ridx") || "",
              version: el.getAttribute("data-rversion") || "",
              tag: el.getAttribute("data-rtag") || "",
              host: el.getAttribute("data-rhost") || ""
            }
          };

          // Include decrypted PASETO payload if available
          if (typeof plaintext === "string") {
            try { detail.decrypted_payload = JSON.parse(plaintext); } catch (_) { detail.decrypted_payload = plaintext; }
          }

          // Include raw bytes as hex if available
          if (typeof innerBytes !== "undefined" && innerBytes instanceof Uint8Array) {
            detail.raw_bytes = Array.prototype.map.call(innerBytes, function(b) {
              return ("0" + b.toString(16)).slice(-2);
            }).join("");
            detail.raw_bytes_length = innerBytes.length;
          }

          var detailJson = JSON.stringify(detail, null, 2);

          // Show clickable [decode error] badge
          el.innerHTML = "";
          var badge = document.createElement("span");
          badge.className = "badge bg-warning text-dark";
          badge.style.cursor = "pointer";
          badge.textContent = "[decode error]";
          badge.title = "Click to view record details";
          badge.addEventListener("click", function() {
            var contentEl = document.getElementById("errorDetailContent");
            if (contentEl) contentEl.textContent = detailJson;
            var modalEl = document.getElementById("errorDetailModal");
            if (modalEl) {
              var modal = new bootstrap.Modal(modalEl);
              modal.show();
            }
          });
          el.appendChild(badge);
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
