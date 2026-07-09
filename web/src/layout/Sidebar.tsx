import { useState, useEffect } from "react";
import { useRouter, type Route } from "../router";
import { useWallet } from "../context/WalletContext";
import { api } from "../api";

/* ─── Navigation Group Structure ─── */

interface NavItem {
	route: Route;
	label: string;
}

interface NavGroup {
	label: string;
	icon: string;
	items: NavItem[];
	defaultOpen?: boolean;
}

const NAV_GROUPS: NavGroup[] = [
	{
		label: "Overview",
		icon: "📊",
		defaultOpen: true,
		items: [
			{ route: "/", label: "Dashboard" },
			{ route: "/stats", label: "Stats" },
		],
	},
	{
		label: "Trade",
		icon: "🔄",
		defaultOpen: true,
		items: [
			{ route: "/offers", label: "Offers" },
			{ route: "/escrows", label: "Escrows" },
			{ route: "/swap", label: "Swap" },
		],
	},
	{
		label: "Finance",
		icon: "🔒",
		items: [
			{ route: "/vaults", label: "Vaults" },
			{ route: "/subscriptions", label: "Subscriptions" },
		],
	},
	{
		label: "Community",
		icon: "👥",
		items: [
			{ route: "/reputation", label: "Reputation" },
			{ route: "/jury", label: "Jury" },
			{ route: "/settings", label: "Settings" },
		],
	},
	{
		label: "Resources",
		icon: "📚",
		items: [
			{ route: "/blog", label: "Blog" },
			{ route: "/security", label: "Security Deep-Dive" },
			{ route: "/docs", label: "Docs" },
			{ route: "/help", label: "Help" },
			{ route: "/testnet", label: "Testnet Guide" },
		],
	},
	{
		label: "Advanced",
		icon: "⚙️",
		items: [
			{ route: "/merchant", label: "Merchant" },
			{ route: "/tokens", label: "Tokens" },
			{ route: "/tokens/create", label: "Create Token" },
		],
	},
];

/* ─── Quick Actions ─── */

interface QuickAction {
	label: string;
	route: Route;
	icon: string;
}

const QUICK_ACTIONS: QuickAction[] = [
	{ label: "Create Escrow", route: "/escrows", icon: "＋" },
	{ label: "New Offer", route: "/offers", icon: "📋" },
];

export function Sidebar({ open, onClose }: { open: boolean; onClose: () => void }) {
	const { route, navigate } = useRouter();
	const { state, connect, setManualAddress } = useWallet();
	const [showManualInput, setShowManualInput] = useState(false);
	const [manualAddr, setManualAddr] = useState("");
	const [offerCount, setOfferCount] = useState<number | null>(null);

	useEffect(() => {
		api.offers().then((d) => {
			const count = (d.offers || []).filter((o: { status: string }) => o.status === "proposed").length;
			setOfferCount(count);
		}).catch(() => {});
		const timer = setInterval(() => {
			api.offers().then((d) => {
				setOfferCount((d.offers || []).filter((o: { status: string }) => o.status === "proposed").length);
			}).catch(() => {});
		}, 60_000);
		return () => clearInterval(timer);
	}, []);
	const [collapsed, setCollapsed] = useState<Set<string>>(() => {
		// Start with groups collapsed by default except those marked defaultOpen
		const start = new Set<string>();
		for (const g of NAV_GROUPS) {
			if (!g.defaultOpen) start.add(g.label);
		}
		return start;
	});
	const [networkWarning, setNetworkWarning] = useState<string | null>(null);

	function toggleGroup(label: string) {
		setCollapsed((prev) => {
			const next = new Set(prev);
			if (next.has(label)) next.delete(label);
			else next.add(label);
			return next;
		});
	}

	function handleNav(r: Route) {
		navigate(r);
		onClose();
	}

	async function handleConnect() {
		setNetworkWarning(null);
		try {
			await connect();
			if (state.network && state.network !== "testnet-11" && state.network !== "mainnet") {
				setNetworkWarning(state.network);
			}
		} catch {
			// connect() sets error in context
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

				{/* Quick actions */}
				<div className="sidebar-actions">
					{QUICK_ACTIONS.map((a) => (
						<button
							key={a.label}
							type="button"
							className={`sidebar-link ${route === a.route ? "sidebar-link--active" : ""}`}
							onClick={() => handleNav(a.route)}
							style={{ fontSize: "12px", padding: "8px 12px" }}
						>
							<span className="sidebar-link-icon" style={{ fontSize: "14px" }}>
								{a.icon}
							</span>
							<span>{a.label}</span>
						</button>
					))}
				</div>

				<nav className="sidebar-nav">
					{NAV_GROUPS.map((group) => {
						const isOpen = !collapsed.has(group.label);
						const isGroupActive = group.items.some((it) => route === it.route);

						return (
							<div key={group.label} className="sidebar-group">
								<button
									type="button"
									className={`sidebar-group-header ${isGroupActive ? "sidebar-group-header--active" : ""}`}
									onClick={() => toggleGroup(group.label)}
								>
									<span className="sidebar-group-icon">{group.icon}</span>
									<span className="sidebar-group-label">{group.label}</span>
									<span className={`sidebar-group-chevron ${isOpen ? "sidebar-group-chevron--open" : ""}`}>
										▸
									</span>
								</button>
								{isOpen && (
									<div className="sidebar-group-items">
										{group.items.map((item) => (
											<button
												type="button"
												key={item.route}
												className={`sidebar-link ${route === item.route ? "sidebar-link--active" : ""}`}
												onClick={() => handleNav(item.route)}
											>
												<span>{item.label}</span>
												{item.route === "/offers" && offerCount !== null && offerCount > 0 && (
													<span className="sidebar-badge">{offerCount}</span>
												)}
											</button>
										))}
									</div>
								)}
							</div>
						);
					})}
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
