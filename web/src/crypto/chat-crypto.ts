import nacl from "tweetnacl";
import { encodeBase64, decodeBase64 } from "tweetnacl-util";

const P = 2n ** 255n - 19n;

function bytesToBigIntLE(bytes: Uint8Array): bigint {
  let n = 0n;
  for (let i = bytes.length - 1; i >= 0; i--) {
    n = (n << 8n) | BigInt(bytes[i]);
  }
  return n;
}

function bigIntToBytesLE(n: bigint, len: number): Uint8Array {
  const out = new Uint8Array(len);
  for (let i = 0; i < len; i++) {
    out[i] = Number(n & 0xffn);
    n >>= 8n;
  }
  return out;
}

function modPow(a: bigint, e: bigint, m: bigint): bigint {
  let r = 1n;
  let b = a % m;
  let exp = e;
  while (exp > 0n) {
    if (exp & 1n) r = (r * b) % m;
    b = (b * b) % m;
    exp >>= 1n;
  }
  return r;
}

function ed25519ToX25519Public(edPub: Uint8Array): Uint8Array {
  const yBytes = new Uint8Array(edPub);
  yBytes[31] &= 0x7f;
  const y = bytesToBigIntLE(yBytes);
  const one = 1n;
  const num = (one + y) % P;
  const denom = (one - y + P) % P;
  const denomInv = modPow(denom, P - 2n, P);
  const u = (num * denomInv) % P;
  return bigIntToBytesLE(u, 32);
}

function clampX25519Secret(seed: Uint8Array): Uint8Array {
  const s = new Uint8Array(seed);
  s[0] &= 248;
  s[31] &= 127;
  s[31] |= 64;
  return s;
}

export type ChatKeypair = {
  pubkey: Uint8Array;
  secret: Uint8Array;
};

export type EncryptedMessage = {
  ciphertext: string;
  nonce: string;
};

export function generateChatKeypair(): ChatKeypair {
  const seed = nacl.randomBytes(32);
  const edKp = nacl.sign.keyPair.fromSeed(seed);
  return { pubkey: edKp.publicKey, secret: seed };
}

export function deriveSharedSecret(
  mySeed: Uint8Array,
  theirEdPubkey: Uint8Array
): Uint8Array {
  const myX = clampX25519Secret(mySeed);
  const theirX = ed25519ToX25519Public(theirEdPubkey);
  return nacl.scalarMult(myX, theirX);
}

export function encryptMessage(
  sharedSecret: Uint8Array,
  plaintext: string
): EncryptedMessage {
  const nonce = nacl.randomBytes(24);
  const messageBytes = new TextEncoder().encode(plaintext);
  const ciphertext = nacl.secretbox(messageBytes, nonce, sharedSecret);
  if (!ciphertext) throw new Error("Encryption failed");
  return {
    ciphertext: encodeBase64(ciphertext),
    nonce: encodeBase64(nonce),
  };
}

export function decryptMessage(
  sharedSecret: Uint8Array,
  ciphertext: string,
  nonce: string
): string | null {
  try {
    const ct = decodeBase64(ciphertext);
    const n = decodeBase64(nonce);
    const decrypted = nacl.secretbox.open(ct, n, sharedSecret);
    if (!decrypted) return null;
    return new TextDecoder().decode(decrypted);
  } catch {
    return null;
  }
}

function ed25519SecretFromSeed(seed: Uint8Array): Uint8Array {
  const edKp = nacl.sign.keyPair.fromSeed(seed);
  return edKp.secretKey;
}

export function signChatMessage(
  seed: Uint8Array,
  contentEnc: string,
  nonce: string,
  escrowId: string,
  seq: number
): string {
  const msg = new TextEncoder().encode(`${contentEnc}:${nonce}:${escrowId}:${seq}`);
  const hash = nacl.hash(msg);
  const sig = nacl.sign.detached(hash, ed25519SecretFromSeed(seed));
  return encodeBase64(sig);
}

export function verifyChatMessage(
  pubkey: Uint8Array,
  contentEnc: string,
  nonce: string,
  escrowId: string,
  seq: number,
  signature: string
): boolean {
  try {
    const msg = new TextEncoder().encode(`${contentEnc}:${nonce}:${escrowId}:${seq}`);
    const hash = nacl.hash(msg);
    return nacl.sign.detached.verify(hash, decodeBase64(signature), pubkey);
  } catch {
    return false;
  }
}
