import { useEffect, useMemo, useState } from "react";
import { api, type Health, type NetworkInfo, type Stats, type Offer } from "./api";
import { money } from "./helpers";
import type { LoadState } from "./helpers";
import { SectionTitle, Panel } from "./ui";

import { WalletStatus } from "./components/wallet";
import { CreateOfferForm, OfferCard, MyOffersPanel } from "./components/offers";
import {
	CreateEscrowForm,
	SwapForm,
	EscrowActionForm,
	DisputeWithEvidenceForm,
	EscrowLookup,
	MyEscrows,
} from "./components/escrows";
import { CreateVaultForm, VaultLookup, VaultListPanel } from "./components/vaults";
import { JuryPanel, ResolveDisputePanel, VouchPanel } from "./components/jury";
import { LinkTelegramForm } from "./components/identity";
import { CompileCovenantForm } from "./components/compile";
import { ReputationLookup, ReceiptLookup } from "./components/lookup";

/* ─── Main App ─── */
export default function App() {
	const [health, setHealth] = useState<LoadState<Health>>({ loading: true });
	const [network, setNetwork] = useState<LoadState<NetworkInfo>>({
		loading: true,
	});
	const [stats, setStats] = useState<LoadState<Stats>>({ loading: true });
	const [offers, setOffers] = useState<LoadState<Offer[]>>({ loading: true });
	const [activeTab, setActiveTab] = useState<
		| "create-vault"
		| "compile"
		| "create-offer"
		| "create-escrow"
		| "settle"
		| "refund"
		| "swap"
		| "dispute"
		| "cancel"
		| "my-offers"
		| "link-telegram"
		| "jury"
		| "vouch"
		| "resolve-dispute"
		| null
	>(null);

	function loadAll() {
		setHealth({ loading: true });
		setNetwork({ loading: true });
		setStats({ loading: true });
		setOffers({ loading: true });
		void Promise.all([
			api
				.health()
				.then((data) => setHealth({ data, loading: false }))
				.catch((err) => setHealth({ error: err.message, loading: false })),
			api
				.network()
				.then((data) => setNetwork({ data, loading: false }))
				.catch((err) => setNetwork({ error: err.message, loading: false })),
			api
				.stats()
				.then((data) => setStats({ data, loading: false }))
				.catch((err) => setStats({ error: err.message, loading: false })),
			api
				.offers()
				.then((data) => setOffers({ data: data.offers, loading: false }))
				.catch((err) => setOffers({ error: err.message, loading: false })),
		]);
	}

	useEffect(loadAll, []);

	const highlights = useMemo(() => {
		const s = stats.data;
		return [
			["Escrows", s?.total_escrows ?? "—"],
			["Active", s?.active_escrows ?? "—"],
			["Volume", s ? money(s.total_volume_kas) : "—"],
			["Settled", s?.settled_escrows ?? "—"],
		];
	}, [stats.data]);

	function closeTab() {
		setActiveTab(null);
		loadAll();
	}

	const tabPanels: Record<string, { title: string; content: React.ReactNode }> = {
		"create-vault": {
			title: "Create vault",
			content: <CreateVaultForm onDone={closeTab} />,
		},
		compile: {
			title: "Compile covenant",
			content: <CompileCovenantForm onDone={closeTab} />,
		},
		"create-offer": {
			title: "Create offer",
			content: <CreateOfferForm onDone={closeTab} />,
		},
		"create-escrow": {
			title: "Create escrow",
			content: <CreateEscrowForm onDone={closeTab} />,
		},
		settle: {
			title: "Settle escrow",
			content: <EscrowActionForm action="settle" />,
		},
		refund: {
			title: "Refund escrow",
			content: <EscrowActionForm action="refund" />,
		},
		swap: {
			title: "Atomic Swap",
			content: <SwapForm onDone={closeTab} />,
		},
		dispute: {
			title: "Dispute escrow",
			content: <DisputeWithEvidenceForm onDone={closeTab} />,
		},
		cancel: {
			title: "Cancel escrow",
			content: <EscrowActionForm action="cancel" />,
		},
		"link-telegram": {
			title: "Link Telegram",
			content: <LinkTelegramForm onDone={closeTab} />,
		},
		"my-offers": {
			title: "My offers",
			content: <MyOffersPanel />,
		},
		jury: {
			title: "Jury panel",
			content: <JuryPanel />,
		},
		vouch: {
			title: "Vouch",
			content: <VouchPanel onDone={closeTab} />,
		},
		"resolve-dispute": {
			title: "Resolve dispute",
			content: <ResolveDisputePanel onDone={closeTab} />,
		},
	};

	return (
		<main className="app">
			<div
				style={{
					background: "#ff9800",
					color: "#000",
					textAlign: "center",
					padding: "8px",
					fontWeight: "bold",
					fontSize: "14px",
				}}
			>
				⚠️ TESTNET — This is a testnet deployment. Do not use real funds. Get testnet KAS from the{" "}
				<a
					href="https://faucet-tn10.kaspanet.io/"
					target="_blank"
					rel="noopener noreferrer"
					style={{ color: "#000", textDecoration: "underline" }}
				>
					Kaspa Testnet Faucet
				</a>
				.
			</div>
			<div
				style={{
					background: "#1a3a1a",
					border: "1px solid rgba(83,215,105,0.3)",
					borderRadius: "8px",
					padding: "16px",
					marginTop: "8px",
					marginBottom: "8px",
				}}
			>
				<strong>🚀 Getting Started</strong>
				<ol
					style={{
						margin: "8px 0 0 0",
						paddingLeft: "20px",
						fontSize: "13px",
						lineHeight: 1.8,
					}}
				>
					<li>
						Install{" "}
						<a
							href="https://kasware.xyz"
							target="_blank"
							rel="noopener noreferrer"
							style={{ color: "#53d769" }}
						>
							KasWare
						</a>{" "}
						browser extension
					</li>
					<li>
						Get testnet KAS from{" "}
						<a
							href="https://faucet-tn10.kaspanet.io/"
							target="_blank"
							rel="noopener noreferrer"
							style={{ color: "#53d769" }}
						>
							Testnet Faucet
						</a>
					</li>
					<li>Connect your wallet using the button in the header</li>
					<li>Create an offer or escrow below</li>
				</ol>
			</div>
			<header className="hero">
				<div>
					<div
						style={{
							display: "flex",
							alignItems: "center",
							justifyContent: "space-between",
							width: "100%",
							marginBottom: "8px",
						}}
					>
						<div className="brand">Kaspa Escrow</div>
						<WalletStatus />
					</div>
					<h1>Trustless escrow and atomic swaps on Kaspa.</h1>
					<p>The public front door for offers, escrows, reputation, and receipts.</p>
				</div>
				<div className="hero-actions">
					<a href="#offers" className="button primary">
						Browse offers
					</a>
					<a href="#actions" className="button">
						Take action
					</a>
				</div>
			</header>

			<section className="grid cards">
				{highlights.map(([label, value]) => (
					<article key={label} className="card">
						<span>{label}</span>
						<strong>{value}</strong>
					</article>
				))}
			</section>

			<section className="grid two-up">
				<Panel title="Network">
					{health.error || network.error ? (
						<p className="muted">{health.error || network.error}</p>
					) : (
						<div className="stack">
							<div className="row">
								<span>API</span>
								<strong>{health.data?.status ?? "—"}</strong>
							</div>
							<div className="row">
								<span>Network</span>
								<strong>{network.data?.network ?? "—"}</strong>
							</div>
							<div className="row">
								<span>Version</span>
								<strong>{health.data?.version ?? "—"}</strong>
							</div>
							<div className="row">
								<span>Fee tier</span>
								<strong>0.5%</strong>
							</div>
						</div>
					)}
				</Panel>

				<Panel title="Public stats">
					{stats.data ? (
						<div className="stack">
							<div className="row">
								<span>Total escrows</span>
								<strong>{stats.data.total_escrows}</strong>
							</div>
							<div className="row">
								<span>Settled</span>
								<strong>{stats.data.settled_escrows}</strong>
							</div>
							<div className="row">
								<span>Disputed</span>
								<strong>{stats.data.disputed_escrows}</strong>
							</div>
							<div className="row">
								<span>Fees</span>
								<strong>{money(stats.data.total_fees_collected_kas)}</strong>
							</div>
						</div>
					) : (
						<p className="muted">Loading stats…</p>
					)}
				</Panel>
			</section>

			<section id="offers">
				<SectionTitle title="Open offers" subtitle="Public listings available to counterparties." />
				<div className="offers">
					{offers.loading && <p className="muted">Loading offers…</p>}
					{offers.error && <p className="muted error-text">{offers.error}</p>}
					{offers.data?.length === 0 && (
						<p className="muted">No open offers right now. Create one below!</p>
					)}
					{offers.data?.map((offer) => (
						<OfferCard key={offer.id} offer={offer} onMutated={loadAll} />
					))}
				</div>
			</section>

			<section id="actions" className="actions-section">
				<SectionTitle
					title="Actions"
					subtitle="Create offers & escrows, settle, refund, dispute, or cancel."
				/>

				<div className="action-tabs">
					<div className="action-group">
						<span className="action-group-label">Create</span>
						{(
							[
								["Offer", "create-offer"],
								["Escrow", "create-escrow"],
								["Vault", "create-vault"],
							] as const
						).map(([label, key]) => (
							<button
								key={key}
								type="button"
								className={`button ${activeTab === key ? "primary" : ""}`}
								onClick={() => setActiveTab(activeTab === key ? null : (key as typeof activeTab))}
							>
								{label}
							</button>
						))}
					</div>
					<div className="action-group">
						<span className="action-group-label">Manage</span>
						{(
							[
								["Settle", "settle"],
								["Refund", "refund"],
								["Swap", "swap"],
								["Dispute", "dispute"],
								["Cancel", "cancel"],
							] as const
						).map(([label, key]) => (
							<button
								key={key}
								type="button"
								className={`button ${activeTab === key ? "primary" : ""}`}
								onClick={() => setActiveTab(activeTab === key ? null : (key as typeof activeTab))}
							>
								{label}
							</button>
						))}
					</div>
					<div className="action-group">
						<span className="action-group-label">Account</span>
						{(
							[
								["My offers", "my-offers"],
								["Telegram", "link-telegram"],
								["Jury", "jury"],
								["Compile", "compile"],
								["Vouch", "vouch"],
								["Resolve", "resolve-dispute"],
							] as const
						).map(([label, key]) => (
							<button
								key={key}
								type="button"
								className={`button ${activeTab === key ? "primary" : ""}`}
								onClick={() => setActiveTab(activeTab === key ? null : (key as typeof activeTab))}
							>
								{label}
							</button>
						))}
					</div>
				</div>

				{activeTab && (
					<div className="panel action-panel">
						<div className="panel-head">
							<h3>{tabPanels[activeTab].title}</h3>
							<button className="button" type="button" onClick={closeTab}>
								✕
							</button>
						</div>
						{tabPanels[activeTab].content}
					</div>
				)}
			</section>

			<section className="grid lookup-grid lookup-section">
				<EscrowLookup />
				<MyEscrows />
				<VaultLookup />
				<VaultListPanel />
				<ReputationLookup />
				<ReceiptLookup />
			</section>
		</main>
	);
}
