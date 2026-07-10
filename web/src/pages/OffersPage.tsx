import { useState, useEffect } from "react";
import { api, type Offer, type TokenRegistryEntry } from "../api";
import { money, sompi, relativeTime, badge } from "../helpers";
import type { LoadState } from "../helpers";
import { useWallet, useAddress } from "../context/WalletContext";
import { ExplorerAddressLink } from "../components/ExplorerLink";
import { useToast } from "../layout/Toast";
import { FormField, SkeletonOffers, SkeletonTable } from "../ui";
import { EmptyState } from "../components/empty-state";
import { Helmet } from "react-helmet-async";
import { useQuery } from "@tanstack/react-query";

type Tab = "browse" | "my-offers" | "create";

export function OffersPage() {
	const [tab, setTab] = useState<Tab>(() => {
		const params = new URLSearchParams(window.location.search);
		return params.has("create") ? "create" : "browse";
	});
	const [presetAsset, setPresetAsset] = useState<string | null>(() => {
		const params = new URLSearchParams(window.location.search);
		return params.get("asset");
	});
	const address = useAddress();
	const { state: wallet } = useWallet();

	return (
		<>
			<Helmet>
				<title>Offer Board — DagLock</title>
				<meta
					name="description"
					content="Browse open escrow offers, find counterparties for KAS and KRC-20 trades on Kaspa."
				/>
				<link rel="canonical" href="https://daglock.com/offers" />
			</Helmet>
			<div>
				<div className="page-header">
					<h1>Offers</h1>
					<p>Browse public trade offers or create your own.</p>
				</div>

				<div className="tab-bar">
					<button
						className={`tab-btn ${tab === "browse" ? "tab-btn--active" : ""}`}
						onClick={() => setTab("browse")}
					>
						Browse
					</button>
					<button
						className={`tab-btn ${tab === "my-offers" ? "tab-btn--active" : ""}`}
						onClick={() => setTab("my-offers")}
					>
						My Offers
					</button>
					<button
						className={`tab-btn ${tab === "create" ? "tab-btn--active" : ""}`}
						onClick={() => setTab("create")}
					>
						Create
					</button>
				</div>

				{/* Offers info banner */}
				<div className="panel" style={{ marginBottom: "16px", padding: "12px 16px" }}>
					<p
						style={{
							margin: 0,
							fontSize: "13px",
							color: "var(--color-text-secondary)",
							lineHeight: 1.5,
						}}
					>
						<strong>How offers work:</strong> Creating an offer doesn't lock funds — it's just a
						listing. When someone <strong>accepts</strong> your offer, an escrow is created and the
						buyer must send KAS to lock it. KRC-20 token trades (
						<em>KRC20:NACHO, KRC20:GHOST, KRC20:KASPY</em> — available after Toccata activates) use
						atomic swaps with a hash preimage — the buyer sends KAS, the seller reveals the secret
						to claim both. Learn more on the{" "}
						<a href="/docs" style={{ color: "var(--color-primary)", textDecoration: "underline" }}>
							Docs page
						</a>
						.
					</p>
				</div>

				{tab === "browse" && <BrowseOffers />}
				{tab === "my-offers" &&
					(wallet.connected ? <MyOffers address={address!} /> : <ConnectPrompt />)}
				{tab === "create" &&
					(wallet.connected ? <CreateOffer address={address!} presetAsset={presetAsset} /> : <ConnectPrompt />)}
			</div>
		</>
	);
}

function ConnectPrompt() {
	const { connect } = useWallet();
	return (
		<EmptyState
			icon="🔗"
			title="Connect your wallet"
			description="Connect KasWare to create and manage offers."
			action={{ label: "Connect Wallet", onClick: connect }}
		/>
	);
}

/* ─── Current KAS/USD price (for USD display) ─── */
function useKasUsdPrice() {
	const { data } = useQuery({
		queryKey: ["kas-usd-price-offers"],
		queryFn: () => api.networkPrice(),
		staleTime: 60_000,
		refetchInterval: 120_000,
	});
	return data?.kas_usd ?? null;
}

function formatUsd(sompi: number, priceUsd: number | null): string {
	if (!priceUsd || !sompi) return "";
	const kas = sompi / 100_000_000;
	return `~$${(kas * priceUsd).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

/* ─── Browse Offers ─── */
function BrowseOffers() {
	const [offers, setOffers] = useState<LoadState<Offer[]>>({ loading: true });
	const [filtered, setFiltered] = useState<Offer[]>([]);
	const [dealTypeFilter, setDealTypeFilter] = useState("all");
	const address = useAddress();
	const usdPrice = useKasUsdPrice();

	useEffect(() => {
		api
			.offers()
			.then((d) => {
				setOffers({ data: d.offers, loading: false });
				setFiltered(d.offers);
			})
			.catch((e) => setOffers({ error: e.message, loading: false }));
	}, []);

	if (offers.loading) return <SkeletonOffers />;
	if (offers.error) return <p className="muted error-text">{offers.error}</p>;

	return (
		<div>
			<div style={{ display: "flex", gap: "6px", alignItems: "center", marginBottom: "12px" }}>
				<span style={{ fontSize: "12px", color: "#888" }}>Deal type:</span>
				{["all", "goods", "otc", "service", "custom"].map((f) => (
					<button
						key={f}
						className={`button ${dealTypeFilter === f ? "primary" : ""}`}
						onClick={() => setDealTypeFilter(f)}
						style={{ fontSize: "11px", padding: "2px 8px" }}
					>
						{f === "all"
							? "All"
							: f === "goods"
								? "🛒 Goods"
								: f === "otc"
									? "🤝 OTC"
									: f === "service"
										? "🛠️ Service"
										: "⚙️ Custom"}
					</button>
				))}
			</div>
			{filtered.length === 0 && (
				<EmptyState icon="📋" title="No open offers" description="Be the first to create one!" />
			)}
			<div className="offers">
				{filtered
					.filter((o) => o.status === "proposed")
					.filter((o) => dealTypeFilter === "all" || o.deal_type === dealTypeFilter)
					.map((o) => (
						<OfferCard
							key={o.id}
							offer={o}
							currentAddress={address}
							usdPrice={usdPrice}
							onMutated={() => {
								api.offers().then((d) => setFiltered(d.offers));
							}}
						/>
					))}
			</div>
		</div>
	);
}

/** Colors for deal_type badges. */
function dealTypeColor(type: string): { bg: string; fg: string; border: string } {
	switch (type) {
		case "otc": return { bg: "#ba68c822", fg: "#ba68c8", border: "#ba68c844" };
		case "goods": return { bg: "#53d76922", fg: "#53d769", border: "#53d76944" };
		case "service": return { bg: "#42a5f522", fg: "#42a5f5", border: "#42a5f544" };
		default: return { bg: "#88888822", fg: "#888888", border: "#88888844" };
	}
}

/** Infer offer type from asset pair for display badge. */
function offerTypeBadge(base: string, quote: string): { label: string; color: string } | null {
	if (base === "KAS" && quote.startsWith("KRC20")) {
		return { label: "Atomic Swap", color: "#ba68c8" };
	}
	if (base === "KAS" && quote === "KAS") {
		return { label: "KAS Escrow", color: "#53d769" };
	}
	return null;
}

/* ─── Offer Card with inline actions ─── */
function OfferCard({
	offer,
	currentAddress,
	usdPrice,
	onMutated,
}: {
	offer: Offer;
	currentAddress: string | null;
	usdPrice: number | null;
	onMutated: () => void;
}) {
	const [loading, setLoading] = useState(false);
	const [counterparty, setCounterparty] = useState(currentAddress || "");
	const [showCounter, setShowCounter] = useState(false);
	const [counterAmount, setCounterAmount] = useState("");
	const [counterMsg, setCounterMsg] = useState("");
	const [counterCount, setCounterCount] = useState<number | null>(null);
	const { notify } = useToast();
	const { sign } = useWallet();

	useEffect(() => {
		api
			.listCounters(offer.id)
			.then((d) => setCounterCount(d.total))
			.catch(() => {});
	}, [offer.id]);

	async function handleAccept() {
		if (!counterparty.startsWith("kaspa:") && !counterparty.startsWith("kaspatest:")) {
			notify("error", "Invalid address", "Enter a valid Kaspa address (kaspa: or kaspatest:)");
			return;
		}
		setLoading(true);
		try {
			const message = `accept:offer:${offer.id}`;
			const signature = await sign(message);
			const auth = { address: counterparty, signature, message };
			await api.acceptOffer(offer.id, counterparty, auth);
			notify("success", "Offer accepted");
			onMutated();
		} catch (e) {
			notify("error", "Failed to accept", (e as Error).message);
		} finally {
			setLoading(false);
		}
	}

	async function handleCancel() {
		if (!currentAddress) return;
		setLoading(true);
		try {
			const message = `cancel:offer:${offer.id}`;
			const signature = await sign(message);
			const auth = { address: currentAddress, signature, message };
			await api.cancelOffer(offer.id, auth);
			notify("success", "Offer cancelled");
			onMutated();
		} catch (e) {
			notify("error", "Failed to cancel", (e as Error).message);
		} finally {
			setLoading(false);
		}
	}

	async function handleCounter(e: React.FormEvent) {
		e.preventDefault();
		const amountNum = Number.parseFloat(counterAmount);
		if (!amountNum || amountNum <= 0) return;
		setLoading(true);
		try {
			await api.counterOffer(offer.id, {
				amount_sompi: sompi(amountNum),
				message: counterMsg || undefined,
			});
			notify("success", "Counter-offer submitted!");
			setShowCounter(false);
			setCounterAmount("");
			setCounterMsg("");
			setCounterCount((c) => (c || 0) + 1);
		} catch (err) {
			notify("error", "Failed", (err as Error).message);
		} finally {
			setLoading(false);
		}
	}

	const canAct = offer.status === "proposed";
	const typeBadge = offerTypeBadge(offer.base_asset, offer.quote_asset);
	const isOwn = currentAddress === offer.creator_address;

	return (
		<article className="offer">
			<div className="offer-top">
				<strong>
					{offer.side.toUpperCase()} {money(offer.amount_sompi)}
					{usdPrice && (
						<span style={{ fontSize: "11px", color: "#888", marginLeft: "6px", fontWeight: 400 }}>
							({formatUsd(offer.amount_sompi, usdPrice)})
						</span>
					)}
				</strong>
				<div style={{ display: "flex", gap: "6px", alignItems: "center" }}>
					{typeBadge && (
						<span
							className="pill"
							style={{
								background: `${typeBadge.color}22`,
								color: typeBadge.color,
								border: `1px solid ${typeBadge.color}44`,
								fontSize: "11px",
							}}
						>
							{typeBadge.label}
						</span>
					)}
					{offer.creator_type === "bot" && (
						<span
							className="pill"
							style={{
								background: "#9c27b022",
								color: "#ce93d8",
								border: "1px solid #9c27b044",
								fontSize: "11px",
							}}
							title="This offer was created by an automated trading bot"
						>
							🤖 Bot
						</span>
					)}
					<span className={badge(offer.status)}>{offer.status}</span>
				</div>
			</div>
			<p>
				{offer.base_asset} for {offer.quote_asset}
			</p>
			{offer.price_type === "market" && offer.current_price && (
				<small className="muted">Market price: ${offer.current_price.toFixed(4)} USD</small>
			)}
			<small className="muted addr">by {offer.creator_address.slice(0, 24)}…</small>
			<code>{offer.id}</code>
			<div style={{ display: "flex", gap: "6px", flexWrap: "wrap", alignItems: "center" }}>
				{offer.deal_type && offer.deal_type !== "custom" && (
					<span
						className="pill"
						style={{
							background: dealTypeColor(offer.deal_type).bg,
							color: dealTypeColor(offer.deal_type).fg,
							border: `1px solid ${dealTypeColor(offer.deal_type).border}`,
							fontSize: "11px",
						}}
					>
						{offer.deal_type === "otc" ? "🤝 OTC" : offer.deal_type === "goods" ? "🛒 Goods" : offer.deal_type === "service" ? "🛠️ Service" : "⚙️ " + offer.deal_type}
					</span>
				)}
				{offer.expires_at && (
					<small className="muted" style={{ fontSize: "11px" }}>
						⏳ {relativeTime(offer.expires_at)} left
					</small>
				)}
			</div>
			{offer.memo && (
				<p style={{ fontSize: "13px", fontStyle: "italic", color: "var(--color-text-secondary)", margin: "4px 0" }}>
					{offer.memo}
				</p>
			)}
			<small className="muted">{relativeTime(offer.created_at)}</small>

			{canAct && (
				<>
					{/* Accept/Cancel for offer creator */}
					{isOwn ? (
						<div className="offer-actions">
							<button className="button" disabled={loading} onClick={handleCancel}>
								Cancel
							</button>
							{counterCount != null && counterCount > 0 && (
								<span className="muted" style={{ fontSize: "12px", marginLeft: "8px" }}>
									{counterCount} counter{counterCount > 1 ? "s" : ""}
								</span>
							)}
						</div>
					) : (
						<>
							{/* Accept for others */}
							<div className="offer-actions">
								<input
									value={counterparty}
									onChange={(e) => setCounterparty(e.target.value)}
									placeholder="your address"
									className="offer-input"
								/>
								<button className="button primary" disabled={loading} onClick={handleAccept}>
									Accept
								</button>
								<button
									className="button"
									disabled={loading}
									onClick={() => setShowCounter(!showCounter)}
								>
									Counter{counterCount != null && counterCount > 0 ? ` (${counterCount})` : ""}
								</button>
							</div>

							{/* Counter-offer form */}
							{showCounter && (
								<form
									className="form form-stacked"
									onSubmit={handleCounter}
									style={{ marginTop: "8px" }}
								>
									<div className="form-field">
										<label style={{ fontSize: "12px" }}>Counter amount (KAS)</label>
										<input
											type="number"
											step="any"
											value={counterAmount}
											onChange={(e) => setCounterAmount(e.target.value)}
											placeholder={((offer.amount_sompi || 0) / 1e8).toFixed(2)}
											style={{ fontSize: "13px" }}
										/>
									</div>
									<div className="form-field">
										<label style={{ fontSize: "12px" }}>Message (optional)</label>
										<input
											value={counterMsg}
											onChange={(e) => setCounterMsg(e.target.value)}
											placeholder="e.g. Can you do this amount?"
											style={{ fontSize: "13px" }}
										/>
									</div>
									<button
										className="button primary"
										type="submit"
										disabled={loading}
										style={{ fontSize: "12px", padding: "4px 12px" }}
									>
										{loading ? "Submitting..." : "Submit Counter"}
									</button>
								</form>
							)}
						</>
					)}
				</>
			)}
		</article>
	);
}

/* ─── My Offers ─── */
function MyOffers({ address }: { address: string }) {
	const [offers, setOffers] = useState<LoadState<Offer[]>>({ loading: true });
	const [filter, setFilter] = useState("all");
	const usdPrice = useKasUsdPrice();

	useEffect(() => {
		api
			.offers(address)
			.then((d) => {
				setOffers({ data: d.offers, loading: false });
			})
			.catch((e) => setOffers({ error: e.message, loading: false }));
	}, [address]);

	const filtered = offers.data?.filter((o) => filter === "all" || o.status === filter);

	return (
		<div>
			{offers.loading && <SkeletonOffers />}
			{offers.error && <p className="muted error-text">{offers.error}</p>}
			{filtered && filtered.length > 0 && (
				<div className="action-tabs" style={{ marginBottom: "12px" }}>
					{["all", "proposed", "accepted", "cancelled"].map((f) => (
						<button
							key={f}
							className={`button ${filter === f ? "primary" : ""}`}
							onClick={() => setFilter(f)}
							style={{ fontSize: "11px", padding: "2px 8px" }}
						>
							{f === "all" ? "All" : f.charAt(0).toUpperCase() + f.slice(1)}
						</button>
					))}
				</div>
			)}
			{filtered?.length === 0 && !offers.loading && (
				<EmptyState
					icon="📋"
					title="No offers yet"
					description="Create your first offer to start trading."
				/>
			)}
			{filtered?.map((o) => (
				<OfferCard
					key={o.id}
					offer={o}
					currentAddress={address}
					usdPrice={usdPrice}
					onMutated={() => {
						api.offers(address).then((d) => setOffers({ data: d.offers, loading: false }));
					}}
				/>
			))}
		</div>
	);
}

/* ─── Create Offer ─── */
function CreateOffer({ address, presetAsset }: { address: string; presetAsset?: string | null }) {
	const [side, setSide] = useState("sell");
	const [baseAsset, setBaseAsset] = useState(() => presetAsset || "KAS");
	const [quoteAsset, setQuoteAsset] = useState(() => presetAsset && presetAsset.startsWith("KRC20:") ? "KAS" : "USDC");
	const [amount, setAmount] = useState("");
	const [expireHours, setExpireHours] = useState("72");
	const [priceType, setPriceType] = useState("fixed");
	const [priceOffset, setPriceOffset] = useState("0");
	const [minPrice, setMinPrice] = useState("");
	const [maxPrice, setMaxPrice] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done">("idle");
	const [registeredTokens, setRegisteredTokens] = useState<TokenRegistryEntry[]>([]);
	const { notify } = useToast();
	const { sign } = useWallet();

	// Fetch registered KRC-20 tokens for dynamic dropdown
	useEffect(() => {
		api.registeredTokens().then((d) => setRegisteredTokens(d.tokens)).catch(() => {});
	}, []);

	// Build KRC-20 ticker list: registered tokens + legacy hardcoded tokens
	const krc20Tickers = [
		...new Set([
			...registeredTokens.map((t) => `KRC20:${t.ticker}`),
			"KRC20:NACHO",
			"KRC20:KASPY",
			"KRC20:GHOST",
		]),
	];

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const amountNum = Number.parseFloat(amount);
		if (!amountNum || amountNum <= 0) return;
		setStatus("loading");
		try {
			const message = `create:offer:${address}`;
			const signature = await sign(message);
			const auth = { address, signature, message };
			await api.createOffer({
				creator_address: address,
				side,
				base_asset: baseAsset,
				quote_asset: quoteAsset,
				amount_sompi: sompi(amountNum),
				expires_at: Math.floor(Date.now() / 1000) + (Number.parseInt(expireHours) || 72) * 3600,
				price_type: priceType,
				...(priceType === "market"
					? {
							price_offset: Number.parseFloat(priceOffset) || 0,
							...(minPrice ? { min_price: Number.parseFloat(minPrice) } : {}),
							...(maxPrice ? { max_price: Number.parseFloat(maxPrice) } : {}),
						}
					: {}),
			}, auth);
			notify("success", "Offer created!");
			setStatus("done");
		} catch (e) {
			notify("error", "Failed to create offer", (e as Error).message);
			setStatus("idle");
		}
	}

	if (status === "done") {
		return (
			<EmptyState
				icon="✅"
				title="Offer created!"
				description="It's now visible on the public board."
			/>
		);
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<FormField label="Side">
				<select value={side} onChange={(e) => setSide(e.target.value)}>
					<option value="sell">Sell</option>
					<option value="buy">Buy</option>
				</select>
			</FormField>
			<FormField label="Sell asset">
				<select value={baseAsset} onChange={(e) => setBaseAsset(e.target.value)}>
					<option value="KAS">KAS</option>
					{krc20Tickers.map((t) => (
						<option key={t} value={t}>{t}</option>
					))}
				</select>
			</FormField>
			<FormField label="For asset">
				<select value={quoteAsset} onChange={(e) => setQuoteAsset(e.target.value)}>
					<option value="USDC">USDC</option>
					<option value="KAS">KAS</option>
					{krc20Tickers.map((t) => (
						<option key={t} value={t}>{t}</option>
					))}
				</select>
			</FormField>
			<FormField label={`Amount (${baseAsset})`}>
				<input
					type="number"
					step="any"
					value={amount}
					onChange={(e) => setAmount(e.target.value)}
					placeholder="100"
				/>
			</FormField>
			<div style={{ fontSize: "13px", color: "#88b888", marginTop: "-8px" }}>
				Your address:{" "}
				<code style={{ display: "inline", fontSize: "12px" }}>{address.slice(0, 20)}…</code>
			</div>
			<FormField label="Price type">
				<select value={priceType} onChange={(e) => setPriceType(e.target.value)}>
					<option value="fixed">Fixed price</option>
					<option value="market">Market price (updates every 15 min)</option>
				</select>
			</FormField>
			{priceType === "market" && (
				<>
					<FormField label="Price offset (%)">
						<input
							type="number"
							step="0.1"
							value={priceOffset}
							onChange={(e) => setPriceOffset(e.target.value)}
							placeholder="0"
						/>
					</FormField>
					<FormField label="Min price (USD)">
						<input
							type="number"
							step="0.001"
							value={minPrice}
							onChange={(e) => setMinPrice(e.target.value)}
							placeholder="0.10"
						/>
					</FormField>
					<FormField label="Max price (USD)">
						<input
							type="number"
							step="0.001"
							value={maxPrice}
							onChange={(e) => setMaxPrice(e.target.value)}
							placeholder="0.20"
						/>
					</FormField>
				</>
			)}
			<FormField label="Expires in">
				<select value={expireHours} onChange={(e) => setExpireHours(e.target.value)}>
					<option value="24">24 hours</option>
					<option value="72">3 days</option>
					<option value="168">7 days</option>
					<option value="720">30 days</option>
				</select>
			</FormField>
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Creating…" : "Create Offer"}
			</button>
		</form>
	);
}
