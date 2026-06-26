import { useState } from "react";
import { useWallet } from "../context/WalletContext";
import { signMessage, mockSignature } from "../kasware";

/* ─── Sign With Wallet Button ─── */
export function SignWithWallet({
	message,
	onSignature,
	walletAddress,
}: {
	message: string;
	onSignature: (sig: string) => void;
	walletAddress: string | null;
}) {
	const { state } = useWallet();
	const [signing, setSigning] = useState(false);
	const [error, setError] = useState("");

	async function handleSign() {
		if (!window.kasware && !state.manualMode) {
			setError("KasWare wallet not detected");
			return;
		}
		setSigning(true);
		setError("");
		try {
			if (state.manualMode) {
				// Mock signature for testnet dev mode
				await new Promise((r) => setTimeout(r, 300)); // brief delay so it feels real
				const sig = mockSignature(message);
				onSignature(sig);
			} else {
				const sig = await signMessage(message, "schnorr");
				onSignature(sig);
			}
		} catch (err) {
			setError((err as Error).message || "Signing cancelled");
		} finally {
			setSigning(false);
		}
	}

	return (
		<div>
			<button
				type="button"
				className="button"
				onClick={handleSign}
				disabled={signing}
				style={{ fontSize: "12px", padding: "4px 10px" }}
			>
				{signing
					? "Signing..."
					: state.manualMode
						? "Mock sign (dev mode)"
						: "Sign with Wallet"}
			</button>
			{error && (
				<p className="muted" style={{ fontSize: "12px", color: "#ff7b7b", marginTop: "4px" }}>
					{error}
				</p>
			)}
			{walletAddress && (
				<p className="muted" style={{ fontSize: "11px", marginTop: "2px" }}>
					Signing as {walletAddress.slice(0, 16)}...
				</p>
			)}
			{state.manualMode && (
				<p className="muted" style={{ fontSize: "11px", marginTop: "2px", color: "#b8960f" }}>
					Testnet dev mode — signature is mocked (any hex works with mock auth).
				</p>
			)}
		</div>
	);
}
