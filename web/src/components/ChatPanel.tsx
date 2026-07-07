import { useState, useEffect, useRef, useCallback } from "react";
import { api, type AuthHeaders, type Escrow, type EscrowMessage } from "../api";
import { useWallet } from "../context/WalletContext";
import { loadOrCreateKeypair, saveKeypair } from "../crypto/chat-store";
import { encodeBase64, decodeBase64 } from "tweetnacl-util";
import {
  deriveSharedSecret,
  encryptMessage,
  decryptMessage,
  signChatMessage,
  verifyChatMessage,
  type ChatKeypair,
} from "../crypto/chat-crypto";
import { useToast } from "../layout/Toast";

interface ChatPanelProps {
  escrow: Escrow;
  onMutated: () => void;
}

export function ChatPanel({ escrow, onMutated }: ChatPanelProps) {
  const { state: wallet, sign } = useWallet();
  const { notify } = useToast();
  const address = wallet.address;

  const [messages, setMessages] = useState<EscrowMessage[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [keypair, setKeypair] = useState<ChatKeypair | null>(null);
  const [sharedSecret, setSharedSecret] = useState<Uint8Array | null>(null);
  const [myPubkeySubmitted, setMyPubkeySubmitted] = useState(false);
  const [pubkeySubmitting, setPubkeySubmitting] = useState(false);
  const [decryptedMessages, setDecryptedMessages] = useState<Record<string, string>>({});
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const isBuyer = address === escrow.buyer_address;
  const counterpartyPubkey = isBuyer
    ? escrow.chat_pubkey_seller
    : escrow.chat_pubkey_buyer;
  const mySubmittedPubkey = isBuyer
    ? escrow.chat_pubkey_buyer
    : escrow.chat_pubkey_seller;

  useEffect(() => {
    if (!address) return;
    const kp = loadOrCreateKeypair(escrow.id);
    setKeypair(kp);
  }, [escrow.id, address]);

  const submitPubkey = useCallback(async () => {
    if (!keypair || !address || !wallet.connected || pubkeySubmitting) return;
    setPubkeySubmitting(true);
    try {
      const pubkeyB64 = encodeBase64(keypair.pubkey);
      const msg = `chat_pubkey:${escrow.id}`;
      const sig = await sign(msg);
      const auth: AuthHeaders = { address, signature: sig, message: msg };
      await api.submitChatPubkey(escrow.id, pubkeyB64, auth);
      setMyPubkeySubmitted(true);
      onMutated();
    } catch (e) {
      console.warn("Failed to submit chat pubkey:", e);
    } finally {
      setPubkeySubmitting(false);
    }
  }, [keypair, address, wallet.connected, escrow.id, onMutated, sign, pubkeySubmitting]);

  useEffect(() => {
    if (!keypair || !address) return;
    if (mySubmittedPubkey === encodeBase64(keypair.pubkey)) {
      setMyPubkeySubmitted(true);
    } else if (!mySubmittedPubkey && wallet.connected) {
      submitPubkey();
    }
  }, [keypair, mySubmittedPubkey, address, wallet.connected, submitPubkey]);

  useEffect(() => {
    if (!keypair || !counterpartyPubkey) {
      setSharedSecret(null);
      return;
    }
    try {
      const secret = deriveSharedSecret(
        keypair.secret,
        decodeBase64(counterpartyPubkey),
      );
      setSharedSecret(secret);
    } catch {
      setSharedSecret(null);
    }
  }, [keypair, counterpartyPubkey]);

  const fetchMessages = useCallback(async () => {
    if (!address || !wallet.connected) return;
    try {
      const msg = `list_messages:${escrow.id}`;
      const sig = await sign(msg);
      const auth: AuthHeaders = { address, signature: sig, message: msg };
      const data = await api.listMessages(escrow.id, auth);
      setMessages(data.messages || []);
    } catch {
      // silent
    } finally {
      setLoading(false);
    }
  }, [escrow.id, address, wallet.connected, sign]);

  useEffect(() => {
    fetchMessages();
    pollRef.current = setInterval(fetchMessages, 15_000);
    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, [fetchMessages]);

  useEffect(() => {
    if (!sharedSecret) return;
    const decrypted: Record<string, string> = {};
    for (const m of messages) {
      if (m.content_enc && m.nonce) {
        const plain = decryptMessage(sharedSecret, m.content_enc, m.nonce);
        if (plain !== null) {
          if (m.chat_sig && m.seq && keypair) {
            const senderPubkey = m.sender_address === escrow.buyer_address
              ? escrow.chat_pubkey_buyer
              : escrow.chat_pubkey_seller;
            if (senderPubkey) {
              const valid = verifyChatMessage(
                decodeBase64(senderPubkey),
                m.content_enc,
                m.nonce,
                m.escrow_id,
                m.seq,
                m.chat_sig,
              );
              if (!valid) {
                decrypted[m.id] = "[signature verification failed]";
                continue;
              }
            }
          }
          decrypted[m.id] = plain;
        }
      }
    }
    setDecryptedMessages(decrypted);
  }, [messages, sharedSecret, keypair, escrow]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  async function handleSend(e: React.FormEvent) {
    e.preventDefault();
    if (!input.trim() || !address || !wallet.connected || !keypair) return;

    if (!sharedSecret) {
      notify("error", "Cannot encrypt message — waiting for counterparty's chat key");
      return;
    }

    setSending(true);
    try {
      const enc = encryptMessage(sharedSecret, input.trim());
      const seq = messages.length + 1;
      const chatSig = signChatMessage(
        keypair.secret,
        enc.ciphertext,
        enc.nonce,
        escrow.id,
        seq,
      );
      const msg = `send_message:${escrow.id}`;
      const sig = await sign(msg);
      const auth: AuthHeaders = { address, signature: sig, message: msg };
      await api.sendMessage(escrow.id, {
        content_enc: enc.ciphertext,
        nonce: enc.nonce,
        chat_sig: chatSig,
      }, auth);
      setInput("");
      await fetchMessages();
    } catch (e) {
      notify("error", "Failed to send message", (e as Error).message);
    } finally {
      setSending(false);
    }
  }

  if (!address) {
    return (
      <div className="panel" style={{ marginTop: "12px" }}>
        <p className="muted" style={{ textAlign: "center", padding: "16px" }}>
          Connect your wallet to use encrypted chat.
        </p>
      </div>
    );
  }

  const canChat = sharedSecret !== null;

  return (
    <div className="panel" style={{ marginTop: "12px" }}>
      <div style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        borderBottom: "1px solid #333",
        padding: "8px 12px",
      }}>
        <strong style={{ fontSize: "14px" }}>
          💬 Encrypted Chat
        </strong>
        {!myPubkeySubmitted && (
          <span style={{ fontSize: "11px", color: "#ff9800" }}>
            ⏳ Submitting chat key…
          </span>
        )}
        {myPubkeySubmitted && !canChat && (
          <span style={{ fontSize: "11px", color: "#ff9800" }}>
            ⏳ Waiting for counterparty's chat key…
          </span>
        )}
        {canChat && (
          <span style={{ fontSize: "11px", color: "#4caf50" }}>
            ✓ E2E encrypted
          </span>
        )}
      </div>

      <div
        style={{
          maxHeight: "300px",
          overflowY: "auto",
          padding: "8px 12px",
          display: "flex",
          flexDirection: "column",
          gap: "6px",
        }}
      >
        {loading && <p className="muted" style={{ textAlign: "center", fontSize: "12px" }}>Loading messages…</p>}
        {!loading && messages.length === 0 && (
          <p className="muted" style={{ textAlign: "center", fontSize: "12px", padding: "16px 0" }}>
            No messages yet. Send the first encrypted message!
          </p>
        )}
        {messages.map((m) => {
          const isMine = m.sender_address === address;
          const displayText = m.content_enc
            ? (decryptedMessages[m.id] ?? "[encrypted — key unavailable]")
            : (m.content || "");
          return (
            <div
              key={m.id}
              style={{
                alignSelf: isMine ? "flex-end" : "flex-start",
                maxWidth: "80%",
                background: isMine ? "#1a3a2a" : "#2a2a2a",
                borderRadius: "8px",
                padding: "6px 10px",
                fontSize: "13px",
                wordBreak: "break-word",
              }}
            >
              <div style={{ color: "#ddd" }}>{displayText}</div>
              <div style={{
                fontSize: "10px",
                color: "#666",
                marginTop: "2px",
                textAlign: "right",
              }}>
                {new Date(m.created_at * 1000).toLocaleTimeString()}
              </div>
            </div>
          );
        })}
        <div ref={messagesEndRef} />
      </div>

      <form
        onSubmit={handleSend}
        style={{
          display: "flex",
          gap: "8px",
          borderTop: "1px solid #333",
          padding: "8px 12px",
        }}
      >
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={canChat ? "Type a message…" : "Waiting for encryption key…"}
          disabled={!canChat || sending}
          style={{ flex: 1, padding: "6px 8px", borderRadius: "4px", border: "1px solid #444", background: "#1a1a1a", color: "#ddd" }}
        />
        <button
          className="button primary"
          type="submit"
          disabled={!canChat || sending || !input.trim()}
          style={{ padding: "6px 14px", fontSize: "13px" }}
        >
          {sending ? "…" : "Send"}
        </button>
      </form>
    </div>
  );
}
