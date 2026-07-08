import { useState } from "react";
import { useRouter, type Route } from "../router";
import { useWallet } from "../context/WalletContext";
import { mockSignature } from "../kasware";

interface NavItem {
	route: Route;
	label: string;
}

const NAV_ITEMS: NavItem[] = [
	{ route: "/", label: "Dashboard" },
	{ route: "/offers", label: "Offers" },
	{ route: "/escrows", label: "Escrows" },
	{ route: "/swap", label: "Swap" },
	{ route: "/vaults", label: "Vaults" },
	{ route: "/subscriptions", label: "Subscriptions" },
	{ route: "/reputation", label: "Reputation" },
	{ route: "/jury", label: "Jury" },
	{ route: "/blog", label: "Blog" },
	{ route: "/security", label: "Security Deep-Dive" },
	{ route: "/merchant", label: "Merchant" },
	{ route: "/stats", label: "Stats" },
	{ route: "/docs", label: "Docs" },
	{ route: "/tokens", label: "Tokens" },
	{ route: "/tokens/create", label: "Create Token" },
	{ route: "/testnet", label: "Testnet" },
	{ route: "/settings", label: "Settings" },
	{ route: "/help", label: "Help" },
];

export function Sidebar({ open, onClose }: { open: boolean; onClose: () => void }) {
	const { route, navigate } = useRouter();
	const { state, connect, setManualAddress } = useWallet();
	const [showManualInput, setShowManualInput] = useState(false);
	const [manualAddr, setManualAddr] = useState("");
	const [networkWarning, setNetworkWarning] = useState<string | null>(null);

	function handleNav(r: Route) {
		navigate(r);
		onClose();
	}

	async function handleConnect() {
		setNetworkWarning(null);
		try {
			await connect();
			// Check for network mismatch — KasWare only supports testnet-11
			if (state.network && state.network !== "testnet-11" && state.network !== "mainnet") {
				setNetworkWarning(state.network);
			}
		} catch {
			// connect() sets error in context, nothing extra needed
		}
	}

	function handleManualSubmit() {
		const addr = manualAddr.trim();
		if (!addr.startsWith("kaspa:")) return;
		setManualAddress(addr);
		setShowManualInput(false);
	}

	return (
		<>
			{/* Mobile overlay */}
			{open && (
				<div
					className="sidebar-overlay"
					onClick={onClose}
					role="button"
					tabIndex={0}
					onKeyDown={(e) => {
						if (e.key === "Enter" || e.key === " ") onClose();
					}}
				/>
			)}

			<aside className={`sidebar ${open ? "sidebar--open" : ""}`}>
				<div className="sidebar-brand">
					<div className="sidebar-logo" />
					<div>
						<div className="sidebar-title">DagLock</div>
						<div className="sidebar-subtitle">Trustless Escrow</div>
					</div>
				</div>

				<nav className="sidebar-nav">
					{NAV_ITEMS.map((item) => (
						<button
							type="button"
							key={item.route}
							className={`sidebar-link ${route === item.route ? "sidebar-link--active" : ""}`}
							onClick={() => handleNav(item.route)}
						>
							<span className="sidebar-link-icon">{item.label.charAt(0)}</span>
							<span>{item.label}</span>
						</button>
					))}
				</nav>

				<div className="sidebar-footer">
					{networkWarning && (
						<div
							style={{
								background: "#fff3cd33",
								border: "1px solid #ffc107",
								borderRadius: "6px",
								padding: "8px",
								fontSize: "11px",
								color: "#e6a700",
								marginBottom: "8px",
							}}
						>
							KasWare connected to <strong>{networkWarning}</strong>, but DagLock runs on{" "}
							<strong>testnet-11</strong>.{" "}
							<button
								type="button"
								onClick={() => {
									setShowManualInput(true);
									setNetworkWarning(null);
								}}
								style={{
									background: "none",
									border: "none",
									color: "#e6a700",
									textDecoration: "underline",
									cursor: "pointer",
									fontSize: "11px",
									padding: 0,
								}}
							>
								Switch to manual mode
							</button>
						</div>
					)}
					{state.connected ? (
						<div className="sidebar-wallet">
							<div className="sidebar-wallet-dot" />
							<div className="sidebar-wallet-info">
								<span className="sidebar-wallet-addr">
									{state.address?.slice(0, 10)}…{state.address?.slice(-4)}
								</span>
								<span className="sidebar-wallet-balance">
									{state.manualMode ? "Testnet mode" : `${state.balance} KAS`}
								</span>
							</div>
						</div>
					) : showManualInput ? (
						<div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
							<input
								value={manualAddr}
								onChange={(e) => setManualAddr(e.target.value)}
								placeholder="kaspa:your-address"
								style={{
									fontSize: "12px",
									padding: "6px 8px",
									borderRadius: "6px",
									border: "1px solid #333",
									background: "#1a1a2e",
									color: "#e0e0e0",
									width: "100%",
									boxSizing: "border-box",
								}}
							/>
							<div style={{ display: "flex", gap: "4px" }}>
								<button
									type="button"
									className="sidebar-connect"
									onClick={handleManualSubmit}
									disabled={!manualAddr.trim().startsWith("kaspa:")}
									style={{ flex: 1 }}
								>
									Set address
								</button>
								<button
									type="button"
									className="sidebar-connect"
									onClick={() => setShowManualInput(false)}
									style={{ flex: 0, padding: "6px 10px" }}
								>
									Back
								</button>
							</div>
						</div>
					) : (
						<>
							<button
								type="button"
								className="sidebar-connect"
								onClick={handleConnect}
								disabled={state.loading}
								style={{ marginBottom: "4px" }}
							>
								{state.loading
									? "Connecting…"
									: state.detected
										? "Connect Wallet"
										: "Install KasWare"}
							</button>
							<button
								type="button"
								className="sidebar-manual"
								onClick={() => setShowManualInput(true)}
								style={{
									background: "transparent",
									border: "1px solid #333",
									fontSize: "11px",
								}}
							>
								Use manual mode
							</button>
						</>
					)}
					{state.network && state.network !== "mainnet" && (
						<div className="sidebar-network"> {state.network}</div>
					)}
					<button
						type="button"
						className="sidebar-manual"
						onClick={() => {
							localStorage.removeItem("daglock_onboarded");
							window.location.reload();
						}}
						style={{
							background: "transparent",
							border: "1px solid #333",
							fontSize: "11px",
							marginTop: "4px",
						}}
					>
						Show tour
					</button>
				</div>
			</aside>
		</>
	);
}
