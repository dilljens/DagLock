import { useEffect, useState } from "react";
import { detectKasware, connectWallet, signMessage, type WalletState } from "../kasware";

/* ─── Wallet Button ─── */
export function WalletStatus() {
	const [wallet, setWallet] = useState<WalletState>({
		detected: false,
		connected: false,
		address: null,
		network: null,
		balance: null,
		loading: false,
		error: null,
	});

	useEffect(() => {
		detectKasware().then((detected) => setWallet((s) => ({ ...s, detected })));
	}, []);

	async function handleConnect() {
		setWallet((s) => ({ ...s, loading: true, error: null }));
		try {
			const { address, network, balance } = await connectWallet();
			setWallet({
				detected: true,
				connected: true,
				address,
				network,
				balance,
				loading: false,
				error: null,
			});
		} catch (err) {
			setWallet((s) => ({
				...s,
				loading: false,
				error: (err as Error).message,
			}));
		}
	}

	return (
		<div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
			{!wallet.detected && (
				<small className="muted" style={{ fontSize: "12px" }}>
					No wallet
				</small>
			)}
			{wallet.detected && !wallet.connected && (
				<button
					type="button"
					className="button"
					onClick={handleConnect}
					disabled={wallet.loading}
					style={{ fontSize: "12px", padding: "4px 10px" }}
				>
					{wallet.loading ? "Connecting..." : "Connect Wallet"}
				</button>
			)}
			{wallet.connected && wallet.address && (
				<small className="muted" style={{ fontSize: "12px" }}>
					{wallet.address.slice(0, 10)}... | {wallet.balance} KAS
				</small>
			)}
		</div>
	);
}

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
	const [signing, setSigning] = useState(false);
	const [error, setError] = useState("");

	async function handleSign() {
		if (!window.kasware) {
			setError("KasWare wallet not detected");
			return;
		}
		setSigning(true);
		setError("");
		try {
			const sig = await signMessage(message, "schnorr");
			onSignature(sig);
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
				{signing ? "Signing..." : "✍️ Sign with Wallet"}
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
		</div>
	);
}
