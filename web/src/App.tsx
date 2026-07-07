import { useState, useEffect, lazy, Suspense } from "react";
import { QueryClient, QueryClientProvider, useQuery } from "@tanstack/react-query";
import { HelmetProvider } from "react-helmet-async";
import { AnimatePresence, motion } from "motion/react";
import * as Tooltip from "@radix-ui/react-tooltip";
import { useRouter, RouterProvider } from "./router";
import { useWallet, WalletProvider } from "./context/WalletContext";
import { ToastProvider } from "./layout/Toast";
import { Sidebar } from "./layout/Sidebar";
import { Footer } from "./layout/Footer";

import { ErrorBoundary } from "./components/error-boundary";
const Dashboard = lazy(() => import("./pages/Dashboard").then((m) => ({ default: m.Dashboard })));
const OffersPage = lazy(() =>
	import("./pages/OffersPage").then((m) => ({ default: m.OffersPage })),
);
const EscrowsPage = lazy(() =>
	import("./pages/EscrowsPage").then((m) => ({ default: m.EscrowsPage })),
);
const VaultsPage = lazy(() =>
	import("./pages/VaultsPage").then((m) => ({ default: m.VaultsPage })),
);
const ReputationPage = lazy(() =>
	import("./pages/ReputationPage").then((m) => ({ default: m.ReputationPage })),
);
const JuryPage = lazy(() => import("./pages/JuryPage").then((m) => ({ default: m.JuryPage })));
const SwapPage = lazy(() => import("./pages/SwapPage").then((m) => ({ default: m.SwapPage })));
const DocsPage = lazy(() => import("./pages/DocsPage").then((m) => ({ default: m.DocsPage })));
const HelpPage = lazy(() => import("./pages/HelpPage").then((m) => ({ default: m.HelpPage })));
const PayInvoicePage = lazy(() =>
	import("./pages/PayInvoicePage").then((m) => ({ default: m.PayInvoicePage })),
);
const TokenDetailPage = lazy(() =>
	import("./pages/TokenDetailPage").then((m) => ({ default: m.TokenDetailPage })),
);
const TokensPage = lazy(() => import("./pages/TokensPage").then((m) => ({ default: m.TokensPage })));
const CreateTokenPage = lazy(() => import("./pages/CreateTokenPage").then((m) => ({ default: m.CreateTokenPage })));
const TestnetPage = lazy(() => import("./pages/TestnetPage").then((m) => ({ default: m.TestnetPage })));
const SettingsPage = lazy(() => import("./pages/SettingsPage").then((m) => ({ default: m.SettingsPage })));
import { OnboardingModal } from "./components/OnboardingModal";
import { FeeCalculator } from "./components/FeeCalculator";

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
	// biome-ignore lint/correctness/useExhaustiveDependencies: route used intentionally to close sidebar on navigation
	useEffect(() => setSidebarOpen(false), [route]);

	const pageContent = (
		<>
			<OnboardingModal />
			<Suspense
				fallback={
					<div className="loading" style={{ textAlign: "center", padding: "3rem", color: "#888" }}>
						Loading…
					</div>
				}
			>
				<ErrorBoundary key={route}>
					{(() => {
					// Dynamic routes: /pay/:id and /swap/:id
					if (window.location.pathname.startsWith("/pay/")) {
						return <PayInvoicePage />;
					}
					if (window.location.pathname.startsWith("/swap/")) {
						return <SwapPage />;
					}
					if (window.location.pathname.startsWith("/tokens/") && !window.location.pathname.startsWith("/tokens/create")) {
						return <TokenDetailPage ticker={window.location.pathname.replace("/tokens/", "")} />;
					}
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
						case "/docs":
							return <DocsPage />;
						case "/help":
							return <HelpPage />;
						case "/tokens":
							return <TokensPage />;
						case "/tokens/create":
							return <CreateTokenPage />;
						case "/testnet":
							return <TestnetPage />;
						case "/settings":
							return <SettingsPage />;
						default:
							return <Dashboard stats={stats} />;
					}
				})()}
			</ErrorBoundary>
		</Suspense>
		</>
	);

	return (
		<div className="app-shell">
			<Sidebar open={sidebarOpen} onClose={() => setSidebarOpen(false)} />
			<main className="main-content">
				<div className="mobile-header">
					<button
						type="button"
						className="hamburger"
						onClick={() => setSidebarOpen(true)}
						aria-label="Open menu"
					>
						☰ <span style={{ fontSize: "14px", marginLeft: "4px" }}>Menu</span>
					</button>
					<span className="brand">DagLock</span>
				</div>

				{/* Testnet banner */}
				{(!wallet.network || wallet.network === "testnet-10") && (
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
							href="https://faucet-testnet.kaspanet.io/"
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
				<Footer />
			</main>
		</div>
	);
}

/* ─── Top-level App ─── */
export default function App() {
	return (
		<HelmetProvider>
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
		</HelmetProvider>
	);
}
