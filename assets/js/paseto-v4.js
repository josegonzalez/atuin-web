// PASETO V4 Local decryption + PASERK PIE key unwrapping
// Implements XChaCha20 stream cipher + BLAKE2b MAC
// Requires blake2b.min.js to be loaded first
(function(global) {
  "use strict";

  // --- Base64url (RFC 4648, no padding) ---

  function base64urlDecode(str) {
    // Convert base64url to standard base64
    var b64 = str.replace(/-/g, "+").replace(/_/g, "/");
    // Add padding
    while (b64.length % 4 !== 0) b64 += "=";
    var raw = atob(b64);
    var out = new Uint8Array(raw.length);
    for (var i = 0; i < raw.length; i++) out[i] = raw.charCodeAt(i);
    return out;
  }

  function base64urlEncode(bytes) {
    var str = "";
    for (var i = 0; i < bytes.length; i++) str += String.fromCharCode(bytes[i]);
    return btoa(str).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  }

  // --- LE64 + PAE (Pre-Authentication Encoding) ---

  function le64(n) {
    var buf = new Uint8Array(8);
    for (var i = 0; i < 7; i++) {
      buf[i] = n & 0xff;
      n = Math.floor(n / 256);
    }
    buf[7] = n & 0x7f; // Clear MSB for interop
    return buf;
  }

  function pae(pieces) {
    // Calculate total length
    var totalLen = 8; // count
    for (var i = 0; i < pieces.length; i++) {
      totalLen += 8 + pieces[i].length; // length prefix + data
    }
    var out = new Uint8Array(totalLen);
    var offset = 0;

    // Number of pieces
    out.set(le64(pieces.length), 0);
    offset += 8;

    for (var j = 0; j < pieces.length; j++) {
      var piece = pieces[j];
      out.set(le64(piece.length), offset);
      offset += 8;
      out.set(piece, offset);
      offset += piece.length;
    }
    return out;
  }

  // --- XChaCha20 Stream Cipher ---

  function rotl32(v, n) {
    return ((v << n) | (v >>> (32 - n))) >>> 0;
  }

  function quarterRound(s, a, b, c, d) {
    s[a] = (s[a] + s[b]) >>> 0; s[d] = rotl32(s[d] ^ s[a], 16);
    s[c] = (s[c] + s[d]) >>> 0; s[b] = rotl32(s[b] ^ s[c], 12);
    s[a] = (s[a] + s[b]) >>> 0; s[d] = rotl32(s[d] ^ s[a], 8);
    s[c] = (s[c] + s[d]) >>> 0; s[b] = rotl32(s[b] ^ s[c], 7);
  }

  function load32le(buf, off) {
    return (buf[off] | (buf[off + 1] << 8) | (buf[off + 2] << 16) | (buf[off + 3] << 24)) >>> 0;
  }

  function store32le(buf, off, val) {
    buf[off] = val & 0xff;
    buf[off + 1] = (val >>> 8) & 0xff;
    buf[off + 2] = (val >>> 16) & 0xff;
    buf[off + 3] = (val >>> 24) & 0xff;
  }

  // ChaCha20 constants: "expand 32-byte k"
  var SIGMA = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574];

  // Generate one 64-byte ChaCha20 block
  function chacha20Block(key, counter, nonce12) {
    var s = new Uint32Array(16);
    // Constants
    s[0] = SIGMA[0]; s[1] = SIGMA[1]; s[2] = SIGMA[2]; s[3] = SIGMA[3];
    // Key (8 x 32-bit words)
    for (var i = 0; i < 8; i++) s[4 + i] = load32le(key, i * 4);
    // Counter
    s[12] = counter >>> 0;
    // Nonce (3 x 32-bit words from 12-byte nonce)
    s[13] = load32le(nonce12, 0);
    s[14] = load32le(nonce12, 4);
    s[15] = load32le(nonce12, 8);

    // Working copy
    var ws = new Uint32Array(s);

    // 20 rounds (10 double-rounds)
    for (var r = 0; r < 10; r++) {
      // Column rounds
      quarterRound(ws, 0, 4,  8, 12);
      quarterRound(ws, 1, 5,  9, 13);
      quarterRound(ws, 2, 6, 10, 14);
      quarterRound(ws, 3, 7, 11, 15);
      // Diagonal rounds
      quarterRound(ws, 0, 5, 10, 15);
      quarterRound(ws, 1, 6, 11, 12);
      quarterRound(ws, 2, 7,  8, 13);
      quarterRound(ws, 3, 4,  9, 14);
    }

    // Add original state
    var out = new Uint8Array(64);
    for (var j = 0; j < 16; j++) {
      store32le(out, j * 4, (ws[j] + s[j]) >>> 0);
    }
    return out;
  }

  // HChaCha20: derive subkey from key(32) + nonce(16) → 32 bytes
  function hchacha20(key, nonce16) {
    var s = new Uint32Array(16);
    s[0] = SIGMA[0]; s[1] = SIGMA[1]; s[2] = SIGMA[2]; s[3] = SIGMA[3];
    for (var i = 0; i < 8; i++) s[4 + i] = load32le(key, i * 4);
    for (var j = 0; j < 4; j++) s[12 + j] = load32le(nonce16, j * 4);

    // 20 rounds (10 double-rounds)
    for (var r = 0; r < 10; r++) {
      quarterRound(s, 0, 4,  8, 12);
      quarterRound(s, 1, 5,  9, 13);
      quarterRound(s, 2, 6, 10, 14);
      quarterRound(s, 3, 7, 11, 15);
      quarterRound(s, 0, 5, 10, 15);
      quarterRound(s, 1, 6, 11, 12);
      quarterRound(s, 2, 7,  8, 13);
      quarterRound(s, 3, 4,  9, 14);
    }

    // Output: state[0..3] || state[12..15] (the non-key words)
    var out = new Uint8Array(32);
    store32le(out, 0,  s[0]);
    store32le(out, 4,  s[1]);
    store32le(out, 8,  s[2]);
    store32le(out, 12, s[3]);
    store32le(out, 16, s[12]);
    store32le(out, 20, s[13]);
    store32le(out, 24, s[14]);
    store32le(out, 28, s[15]);
    return out;
  }

  // XChaCha20: key(32) + nonce(24) + data → XOR'd output
  function xchacha20(key, nonce24, data) {
    // Step 1: Derive subkey via HChaCha20 using first 16 bytes of nonce
    var subkey = hchacha20(key, nonce24.subarray(0, 16));

    // Step 2: Build 12-byte nonce: 4 zero bytes + last 8 bytes of nonce24
    var nonce12 = new Uint8Array(12);
    nonce12.set(nonce24.subarray(16, 24), 4);

    // Step 3: ChaCha20 with subkey, counter=0, nonce12
    var out = new Uint8Array(data.length);
    var counter = 0;
    var offset = 0;

    while (offset < data.length) {
      var block = chacha20Block(subkey, counter, nonce12);
      var remaining = data.length - offset;
      var take = remaining < 64 ? remaining : 64;
      for (var i = 0; i < take; i++) {
        out[offset + i] = data[offset + i] ^ block[i];
      }
      offset += take;
      counter++;
    }

    return out;
  }

  // --- Constant-time comparison ---

  function constantTimeEqual(a, b) {
    if (a.length !== b.length) return false;
    var diff = 0;
    for (var i = 0; i < a.length; i++) diff |= a[i] ^ b[i];
    return diff === 0;
  }

  // --- Helper: string to Uint8Array ---

  function strToBytes(s) {
    return new TextEncoder().encode(s);
  }

  // --- PASETO V4 Local Decrypt ---

  var V4_HEADER = "v4.local.";

  function v4LocalDecrypt(token, key, implicit) {
    // Validate header
    if (token.substring(0, V4_HEADER.length) !== V4_HEADER) {
      throw new Error("Invalid PASETO V4 header");
    }

    // Split token into payload and optional footer
    var rest = token.substring(V4_HEADER.length);
    var parts = rest.split(".");
    var payloadB64 = parts[0];
    var footerB64 = parts.length > 1 ? parts[1] : "";

    // Decode payload
    var payload = base64urlDecode(payloadB64);
    if (payload.length < 64) {
      throw new Error("Token payload too short");
    }

    // Split: nonce(32) | ciphertext | tag(32)
    var n = payload.subarray(0, 32);
    var t = payload.subarray(payload.length - 32);
    var c = payload.subarray(32, payload.length - 32);

    // Decode footer
    var f = footerB64 ? base64urlDecode(footerB64) : new Uint8Array(0);

    // Implicit assertion
    var i = implicit ? strToBytes(implicit) : new Uint8Array(0);

    // Header as bytes
    var h = strToBytes(V4_HEADER);

    // Derive Ek || n2 (56 bytes) using BLAKE2b keyed hash
    // msg = "paseto-encryption-key" || n, key = k, outlen = 56
    var encMsg = new Uint8Array(21 + 32);
    encMsg.set(strToBytes("paseto-encryption-key"), 0);
    encMsg.set(n, 21);
    var tmp = blake2b(encMsg, key, 56);
    var Ek = tmp.subarray(0, 32);
    var n2 = tmp.subarray(32, 56);

    // Derive Ak (32 bytes)
    // msg = "paseto-auth-key-for-aead" || n, key = k, outlen = 32
    var authMsg = new Uint8Array(24 + 32);
    authMsg.set(strToBytes("paseto-auth-key-for-aead"), 0);
    authMsg.set(n, 24);
    var Ak = blake2b(authMsg, key, 32);

    // Compute expected tag: BLAKE2b(key=Ak, msg=PAE(h, n, c, f, i), outlen=32)
    var preAuth = pae([h, n, c, f, i]);
    var t2 = blake2b(preAuth, Ak, 32);

    // Verify tag
    if (!constantTimeEqual(t, t2)) {
      return null;
    }

    // Decrypt ciphertext with XChaCha20
    var plaintext = xchacha20(Ek, n2, c);

    return new TextDecoder().decode(plaintext);
  }

  // --- PASERK PIE Unwrap (V4 local-wrap) ---

  var PIE_HEADER = "k4.local-wrap.pie.";

  function pieUnwrapV4(wrappedKey, wrappingKey) {
    // Validate header
    if (wrappedKey.substring(0, PIE_HEADER.length) !== PIE_HEADER) {
      throw new Error("Invalid PASERK PIE header");
    }

    // Decode payload after header
    var payloadB64 = wrappedKey.substring(PIE_HEADER.length);
    var payload = base64urlDecode(payloadB64);
    if (payload.length < 64) {
      throw new Error("PIE payload too short");
    }

    // Split: tag(32) | nonce(32) | ciphertext
    var tag = payload.subarray(0, 32);
    var n = payload.subarray(32, 64);
    var c = payload.subarray(64);

    // Derive Ak: BLAKE2b(key=wk, msg=0x81||n, outlen=32)
    var authMsg = new Uint8Array(1 + 32);
    authMsg[0] = 0x81;
    authMsg.set(n, 1);
    var Ak = blake2b(authMsg, wrappingKey, 32);

    // Compute expected tag: BLAKE2b(key=Ak, msg=h||n||c, outlen=32)
    var h = strToBytes(PIE_HEADER);
    var tagMsg = new Uint8Array(h.length + n.length + c.length);
    tagMsg.set(h, 0);
    tagMsg.set(n, h.length);
    tagMsg.set(c, h.length + n.length);
    var t2 = blake2b(tagMsg, Ak, 32);

    // Verify tag
    if (!constantTimeEqual(tag, t2)) {
      return null;
    }

    // Derive Ek || n2: BLAKE2b(key=wk, msg=0x80||n, outlen=56)
    var encMsg = new Uint8Array(1 + 32);
    encMsg[0] = 0x80;
    encMsg.set(n, 1);
    var tmp = blake2b(encMsg, wrappingKey, 56);
    var Ek = tmp.subarray(0, 32);
    var n2 = tmp.subarray(32, 56);

    // Decrypt CEK
    var ptk = xchacha20(Ek, n2, c);

    // V4 local key must be exactly 32 bytes
    if (ptk.length !== 32) {
      return null;
    }

    return ptk;
  }

  // --- Public API ---

  global.PasetoV4 = {
    decrypt: v4LocalDecrypt,
    unwrapPIE: pieUnwrapV4,
    _base64urlDecode: base64urlDecode,
    _base64urlEncode: base64urlEncode
  };

})(typeof window !== "undefined" ? window : typeof globalThis !== "undefined" ? globalThis : this);
