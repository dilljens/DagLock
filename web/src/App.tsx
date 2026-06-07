import { useEffect, useState, useMemo } from "react";
import { useRouter, RouterProvider } from "./router";
import { useWallet, WalletProvider } from "./context/WalletContext";
import { ToastProvider, useToast } from "./layout/Toast";
import { Sidebar } from "./layout/Sidebar";

import { Dashboard } from "./pages/Dashboard";
import { OffersPage } from "./pages/OffersPage";
import { EscrowsPage } from "./pages/EscrowsPage";
import { VaultsPage } from "./pages/VaultsPage";
import { ReputationPage } from "./pages/ReputationPage";
import { JuryPage } from "./pages/JuryPage";

import { api, type Health, type Stats, type Offer } from "./api";
import { money } from "./helpers";
import type { LoadState } from "./helpers";

/* ─── Inner app (has access to router + wallet) ─── */
function AppInner() {
	const { route } = useRouter();
	const { state: wallet } = useWallet();
	const [sidebarOpen, setSidebarOpen] = useState(false);
	const [health, setHealth] = useState<LoadState<Health>>({ loading: true });
	const [stats, setStats] = useState<LoadState<Stats>>({ loading: true });
	const [offers, setOffers] = useState<LoadState<Offer[]>>({ loading: true });

	useEffect(() => {
		void Promise.all([
			api
				.health()
				.then((d) => setHealth({ data: d, loading: false }))
				.catch((e) => setHealth({ error: e.message, loading: false })),
			api
				.stats()
				.then((d) => setStats({ data: d, loading: false }))
				.catch((e) => setStats({ error: e.message, loading: false })),
			api
				.offers()
				.then((d) => setOffers({ data: d.offers, loading: false }))
				.catch((e) => setOffers({ error: e.message, loading: false })),
		]);
	}, []);

	// Close sidebar on route change
	useEffect(() => setSidebarOpen(false), [route]);

	const pageContent = (() => {
		switch (route) {
			case "/":
				return <Dashboard health={health} stats={stats} offers={offers} />;
			case "/offers":
				return <OffersPage />;
			case "/escrows":
				return <EscrowsPage />;
			case "/vaults":
				return <VaultsPage />;
			case "/reputation":
				return <ReputationPage />;
			case "/jury":
				return <JuryPage />;
			default:
				return <Dashboard health={health} stats={stats} offers={offers} />;
		}
	})();

	return (
		<div className="app-shell">
			<Sidebar open={sidebarOpen} onClose={() => setSidebarOpen(false)} />
			<main className="main-content">
				<div className="mobile-header">
					<button className="hamburger" onClick={() => setSidebarOpen(true)}>
						☰
					</button>
					<span className="brand">DagLock</span>
				</div>

				{/* Testnet banner */}
				{(!wallet.network || wallet.network === "testnet-12") && (
					<div
						style={{
							background: "#ff9800",
							color: "#000",
							textAlign: "center",
							padding: "8px 12px",
							fontWeight: 600,
							fontSize: "13px",
							borderRadius: "12px",
							marginBottom: "16px",
						}}
					>
						🧪 TESTNET — Use{" "}
						<a
							href="https://faucet-tn10.kaspanet.io/"
							target="_blank"
							rel="noopener noreferrer"
							style={{ color: "#000", textDecoration: "underline" }}
						>
							Testnet Faucet
						</a>{" "}
						for test KAS. Do not use real funds.
					</div>
				)}

				{pageContent}
			</main>
		</div>
	);
}

/* ─── Top-level App ─── */
export default function App() {
	return (
		<RouterProvider>
			<WalletProvider>
				<ToastProvider>
					<AppInner />
				</ToastProvider>
			</WalletProvider>
		</RouterProvider>
	);
}
