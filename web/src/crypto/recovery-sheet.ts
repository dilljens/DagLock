export interface RecoveryData {
  escrowId: string;
  chatPubkey: string;
  chatSecret: string;
  createdAt: string;
}

export function generateRecoverySheet(data: RecoveryData): string {
  return [
    "=== DagLock Chat Key Recovery Sheet ===",
    "DO NOT SHARE THIS FILE. Anyone with this file can read your encrypted messages.",
    "",
    `Escrow ID: ${data.escrowId}`,
    `Chat Public Key: ${data.chatPubkey}`,
    `Chat Private Key: ${data.chatSecret}  ← KEEP SECRET`,
    `Created: ${data.createdAt}`,
    "",
    "=== Recovery Instructions ===",
    `1. Go to https://daglock.com/escrows/${data.escrowId}`,
    `2. Click \"Restore chat keys\"`,
    "3. Paste the Chat Private Key from this file",
    "4. Your chat will be restored",
    "",
    "=== Security Notes ===",
    "- This chat key CANNOT spend funds. It can only read and send messages.",
    "- Keep this file offline. Delete after restoring on your device.",
    "- If you lose this key, your chat history is unrecoverable.",
    "- DagLock does NOT store your private keys.",
  ].join("\n");
}

export function downloadRecoverySheet(data: RecoveryData): void {
  const content = generateRecoverySheet(data);
  const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `daglock-chat-key-${data.escrowId}.txt`;
  a.click();
  URL.revokeObjectURL(url);
}
