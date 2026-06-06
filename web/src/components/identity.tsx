import { useState } from "react";
import { api, type AuthHeaders } from "../api";
import { FormField } from "../ui";
import { SignWithWallet } from "./wallet";

/* ─── Verify Telegram identity ─── */
export function LinkTelegramForm({ onDone }: { onDone: () => void }) {
	const [address, setAddress] = useState("");
	const [telegramHandle, setTelegramHandle] = useState("");
	const [signature, setSignature] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">("idle");
	const [error, setError] = useState("");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const trimmedTgAddr = address.trim();
		const trimmedTgHandle = telegramHandle.trim();
		if (!trimmedTgAddr.startsWith("kaspa:")) {
			setError("Invalid address format. Must be a valid Kaspa address starting with 'kaspa:'.");
			return;
		}
		if (!trimmedTgHandle.startsWith("@")) {
			setError("Invalid Telegram handle. Must start with '@' (e.g., @username).");
			return;
		}
		if (!signature.trim()) {
			setError(
				"Signature is required for verification. Please sign a message with your Kaspa wallet.",
			);
			return;
		}
		setStatus("loading");
		setError("");
		try {
			const message = `daglock.io:verify:telegram:${trimmedTgHandle}`;
			const auth: AuthHeaders = {
				address: trimmedTgAddr,
				signature: signature.trim(),
				message,
			};
			await api.createIdentity("telegram", trimmedTgHandle, message, signature.trim(), auth);
			setStatus("done");
			onDone();
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	if (status === "done") {
		return <p className="muted success-text">Telegram linked!</p>;
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<p className="muted">Sign a message with your Kaspa wallet. The format is:</p>
			<code>daglock.io:verify:telegram:YOUR_HANDLE</code>
			<FormField label="Your address">
				<input
					value={address}
					onChange={(e) => setAddress(e.target.value)}
					placeholder="kaspa:..."
				/>
			</FormField>
			<FormField label="Telegram handle">
				<input
					value={telegramHandle}
					onChange={(e) => setTelegramHandle(e.target.value)}
					placeholder="@yourhandle"
				/>
			</FormField>
			<FormField label="Signature">
				<div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
					<input
						value={signature}
						onChange={(e) => setSignature(e.target.value)}
						placeholder="auto-filled when signing"
						readOnly={signature.length > 0}
						style={{ flex: 1 }}
					/>
					<SignWithWallet
						message={`daglock.io:verify:telegram:${telegramHandle}`}
						onSignature={(sig) => setSignature(sig)}
						walletAddress={address}
					/>
				</div>
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Verifying…" : "Link Telegram"}
			</button>
		</form>
	);
}
