import { useRouter, type Route } from "../router";
import { useWallet } from "../context/WalletContext";

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
	{ route: "/reputation", label: "Reputation" },
	{ route: "/jury", label: "Jury" },
];

export function Sidebar({ open, onClose }: { open: boolean; onClose: () => void }) {
	const { route, navigate } = useRouter();
	const { state, connect } = useWallet();

	function handleNav(r: Route) {
		navigate(r);
		onClose();
	}

	return (
		<>
			{/* Mobile overlay */}
			{open && <div className="sidebar-overlay" onClick={onClose} />}

			<aside className={`sidebar ${open ? "sidebar--open" : ""}`}>
				<div className="sidebar-brand">
					<div className="sidebar-logo"></div>
					<div>
						<div className="sidebar-title">DagLock</div>
						<div className="sidebar-subtitle">Trustless Escrow</div>
					</div>
				</div>

				<nav className="sidebar-nav">
					{NAV_ITEMS.map((item) => (
						<button
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
					{state.connected ? (
						<div className="sidebar-wallet">
							<div className="sidebar-wallet-dot" />
							<div className="sidebar-wallet-info">
								<span className="sidebar-wallet-addr">
									{state.address?.slice(0, 10)}…{state.address?.slice(-4)}
								</span>
								<span className="sidebar-wallet-balance">{state.balance} KAS</span>
							</div>
						</div>
					) : (
						<button className="sidebar-connect" onClick={connect} disabled={state.loading}>
							{state.detected
								? state.loading
									? "Connecting…"
									: "Connect Wallet"
								: "Install KasWare"}
						</button>
					)}
					{state.network && state.network !== "mainnet" && (
						<div className="sidebar-network"> {state.network}</div>
					)}
				</div>
			</aside>
		</>
	);
}
