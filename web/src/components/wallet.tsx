import { useEffect, useState } from "react";
import { useWallet } from "../context/WalletContext";
import { detectKasware, connectWallet, signMessage, mockSignature, type WalletState } from "../kasware";

/* ─── Wallet Button ─── */
export function WalletStatus({ onConnect }: { onConnect?: (addr: string) => void }) {
	const { setManualAddress } = useWallet();
	const [wallet, setWallet] = useState<WalletState>({
		detected: false,
		connected: false,
		address: null,
		network: null,
		balance: null,
		loading: false,
		error: null,
		manualMode: false,
	});
	const [showManualInput, setShowManualInput] = useState(false);
	const [manualAddr, setManualAddr] = useState("");

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
				manualMode: false,
			});
			onConnect?.(address);
		} catch (err) {
			setWallet((s) => ({
				...s,
				loading: false,
				error: (err as Error).message,
			}));
		}
	}

	function handleManualSubmit() {
		const addr = manualAddr.trim();
		if (!addr.startsWith("kaspa:")) return;
		setWallet({
			detected: false,
			connected: true,
			address: addr,
			network: "testnet-12",
			balance: null,
			loading: false,
			error: null,
			manualMode: true,
		});
		setManualAddress(addr);
		onConnect?.(addr);
	}

	return (
		<div style={{ display: "flex", alignItems: "center", gap: "8px", flexWrap: "wrap" }}>
			{!wallet.detected && !wallet.connected && !showManualInput && (
				<>
					<a
						href="https://kasware.xyz"
						target="_blank"
						rel="noopener noreferrer"
						className="button"
						style={{ fontSize: "12px", padding: "4px 10px", textDecoration: "none" }}
					>
						Install KasWare
					</a>
					<button
						type="button"
						className="button"
						onClick={() => setShowManualInput(true)}
						style={{ fontSize: "12px", padding: "4px 10px" }}
					>
						Continue without wallet
					</button>
				</>
			)}
			{!wallet.detected && showManualInput && (
				<div style={{ display: "flex", gap: "4px", alignItems: "center" }}>
					<input
						value={manualAddr}
						onChange={(e) => setManualAddr(e.target.value)}
						placeholder="kaspa:your-address"
						style={{ fontSize: "12px", padding: "4px 6px", width: "220px" }}
					/>
					<button
						type="button"
						className="button"
						onClick={handleManualSubmit}
						disabled={!manualAddr.trim().startsWith("kaspa:")}
						style={{ fontSize: "12px", padding: "4px 10px" }}
					>
						Set address
					</button>
				</div>
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
				<>
					<small className="muted" style={{ fontSize: "12px" }}>
						{wallet.address.slice(0, 10)}...{wallet.manualMode ? "" : ` | ${wallet.balance} KAS`}
					</small>
					{wallet.manualMode && (
						<span
							style={{
								fontSize: "11px",
								background: "#ffd70033",
								color: "#b8960f",
								padding: "2px 6px",
								borderRadius: "4px",
							}}
						>
							Testnet mode
						</span>
					)}
				</>
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
