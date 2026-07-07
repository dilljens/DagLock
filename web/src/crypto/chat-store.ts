import { generateChatKeypair, type ChatKeypair } from "./chat-crypto";
import { encodeBase64, decodeBase64 } from "tweetnacl-util";

const STORAGE_PREFIX = "daglock_chat_keypair_";

export function loadOrCreateKeypair(escrowId: string): ChatKeypair {
  const existing = loadKeypair(escrowId);
  if (existing) return existing;
  const kp = generateChatKeypair();
  saveKeypair(escrowId, kp);
  return kp;
}

export function loadKeypair(escrowId: string): ChatKeypair | null {
  try {
    const raw = localStorage.getItem(STORAGE_PREFIX + escrowId);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    return {
      pubkey: decodeBase64(parsed.pubkey),
      secret: decodeBase64(parsed.secret),
    };
  } catch {
    return null;
  }
}

export function saveKeypair(escrowId: string, kp: ChatKeypair): void {
  localStorage.setItem(
    STORAGE_PREFIX + escrowId,
    JSON.stringify({
      pubkey: encodeBase64(kp.pubkey),
      secret: encodeBase64(kp.secret),
    })
  );
}

export function chatPubkeyB64(escrowId: string): string | null {
  const kp = loadKeypair(escrowId);
  if (!kp) return null;
  return encodeBase64(kp.pubkey);
}
