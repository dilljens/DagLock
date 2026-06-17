import { useState, useEffect } from "react";
import { QueryClient, QueryClientProvider, useQuery } from "@tanstack/react-query";
import { AnimatePresence, motion } from "motion/react";
import * as Tooltip from "@radix-ui/react-tooltip";
import { useRouter, RouterProvider } from "./router";
import { useWallet, WalletProvider } from "./context/WalletContext";
import { ToastProvider, useToast } from "./layout/Toast";
import { Sidebar } from "./layout/Sidebar";

import { ErrorBoundary } from "./components/error-boundary";
import { Dashboard } from "./pages/Dashboard";
import { OffersPage } from "./pages/OffersPage";
import { EscrowsPage } from "./pages/EscrowsPage";
import { VaultsPage } from "./pages/VaultsPage";
import { ReputationPage } from "./pages/ReputationPage";
import { JuryPage } from "./pages/JuryPage";
import { SwapPage } from "./pages/SwapPage";

import { api } from "./api";
import { useWebSocket } from "./hooks/useWebSocket";

const queryClient = new QueryClient({
	defaultOptions: {
		queries: {
			staleTime: 30_000,
			retry: 2,
			refetchOnWindowFocus: false,
		},
	},
});

/* ─── Inner app (has access to router + wallet) ─── */
function AppInner() {
	const { route } = useRouter();
	const { state: wallet } = useWallet();
	const [sidebarOpen, setSidebarOpen] = useState(false);

	const { data: health, isLoading: healthLoading } = useQuery({
		queryKey: ["health"],
		queryFn: () => api.health(),
	});

	const { data: stats } = useQuery({
		queryKey: ["stats"],
		queryFn: () => api.stats(),
	});

	const { data: offers, isLoading: offersLoading } = useQuery({
		queryKey: ["offers"],
		queryFn: () => api.offers().then((d) => d.offers),
		staleTime: 15_000,
	});

	// Real-time updates via WebSocket
	useWebSocket();

	// Close sidebar on route change
	useEffect(() => setSidebarOpen(false), [route]);

	const pageContent = (() => {
		const page = (() => {
			switch (route) {
				case "/":
					return <Dashboard stats={stats} />;
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
				case "/swap":
					return <SwapPage />;
				default:
					return <Dashboard stats={stats} />;
			}
		})();
		return <ErrorBoundary key={route}>{page}</ErrorBoundary>;
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
						TESTNET — Use{" "}
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

				<AnimatePresence mode="wait">
					<motion.div
						key={route}
						initial={{ opacity: 0, y: 12 }}
						animate={{ opacity: 1, y: 0 }}
						exit={{ opacity: 0, y: -12 }}
						transition={{ duration: 0.15, ease: [0.16, 1, 0.3, 1] }}
					>
						{pageContent}
					</motion.div>
				</AnimatePresence>
			</main>
		</div>
	);
}

/* ─── Top-level App ─── */
export default function App() {
	return (
		<QueryClientProvider client={queryClient}>
			<Tooltip.Provider delayDuration={400}>
				<RouterProvider>
					<WalletProvider>
						<ToastProvider>
							<AppInner />
						</ToastProvider>
					</WalletProvider>
				</RouterProvider>
			</Tooltip.Provider>
		</QueryClientProvider>
	);
}
