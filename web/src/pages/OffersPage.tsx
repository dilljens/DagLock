import { useState } from "react";
import { api, type Offer } from "../api";
import { money, sompi, relativeTime, badge } from "../helpers";
import type { LoadState } from "../helpers";
import { useWallet, useAddress } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { FormField, SkeletonOffers, SkeletonTable } from "../ui";
import { EmptyState } from "../components/empty-state";
import { SignWithWallet } from "../components/wallet";

type Tab = "browse" | "my-offers" | "create";

export function OffersPage() {
	const [tab, setTab] = useState<Tab>("browse");
	const address = useAddress();
	const { state: wallet } = useWallet();

	return (
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

			{tab === "browse" && <BrowseOffers />}
			{tab === "my-offers" &&
				(wallet.connected ? <MyOffers address={address!} /> : <ConnectPrompt />)}
			{tab === "create" &&
				(wallet.connected ? <CreateOffer address={address!} /> : <ConnectPrompt />)}
		</div>
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

/* ─── Browse Offers ─── */
function BrowseOffers() {
	const [offers, setOffers] = useState<LoadState<Offer[]>>({ loading: true });
	const [filtered, setFiltered] = useState<Offer[]>([]);
	const address = useAddress();

	if (offers.loading) {
		api
			.offers()
			.then((d) => {
				setOffers({ data: d.offers, loading: false });
				setFiltered(d.offers);
			})
			.catch((e) => setOffers({ error: e.message, loading: false }));
	}

	if (offers.loading) return <SkeletonOffers />;
	if (offers.error) return <p className="muted error-text">{offers.error}</p>;

	return (
		<div>
			{filtered.length === 0 && (
				<EmptyState
					icon="📋"
					title="No open offers"
					description="Be the first to create one!"
				/>
			)}
			<div className="offers">
				{filtered
					.filter((o) => o.status === "proposed")
					.map((o) => (
						<OfferCard
							key={o.id}
							offer={o}
							currentAddress={address}
							onMutated={() => {
								api.offers().then((d) => setFiltered(d.offers));
							}}
						/>
					))}
			</div>
		</div>
	);
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
	onMutated,
}: {
	offer: Offer;
	currentAddress: string | null;
	onMutated: () => void;
}) {
	const [loading, setLoading] = useState(false);
	const [counterparty, setCounterparty] = useState(currentAddress || "");
	const { notify } = useToast();

	async function handleAccept() {
		if (!counterparty.startsWith("kaspa:")) return;
		setLoading(true);
		try {
			await api.acceptOffer(offer.id, counterparty);
			notify("success", "Offer accepted");
			onMutated();
		} catch (e) {
			notify("error", "Failed to accept", (e as Error).message);
		} finally {
			setLoading(false);
		}
	}

	async function handleCancel() {
		setLoading(true);
		try {
			await api.cancelOffer(offer.id);
			notify("success", "Offer cancelled");
			onMutated();
		} catch (e) {
			notify("error", "Failed to cancel", (e as Error).message);
		} finally {
			setLoading(false);
		}
	}

	const canAct = offer.status === "proposed";
	const typeBadge = offerTypeBadge(offer.base_asset, offer.quote_asset);

	return (
		<article className="offer">
			<div className="offer-top">
				<strong>
					{offer.side.toUpperCase()} {money(offer.amount_sompi)}
				</strong>
				<div style={{ display: "flex", gap: "6px", alignItems: "center" }}>
					{typeBadge && (
						<span className="pill" style={{
							background: `${typeBadge.color}22`,
							color: typeBadge.color,
							border: `1px solid ${typeBadge.color}44`,
							fontSize: "11px",
						}}>
							{typeBadge.label}
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
			<small className="muted">{relativeTime(offer.created_at)}</small>
			{canAct && (
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
					<button className="button" disabled={loading} onClick={handleCancel}>
						Cancel
					</button>
				</div>
			)}
		</article>
	);
}

/* ─── My Offers ─── */
function MyOffers({ address }: { address: string }) {
	const [offers, setOffers] = useState<LoadState<Offer[]>>({ loading: true });
	const [filter, setFilter] = useState("all");

	if (offers.loading) {
		api
			.offers(address)
			.then((d) => {
				setOffers({ data: d.offers, loading: false });
			})
			.catch((e) => setOffers({ error: e.message, loading: false }));
	}

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
				<article key={o.id} className="offer" style={{ cursor: "default" }}>
					<div className="offer-top">
						<strong>
							{o.side.toUpperCase()} {money(o.amount_sompi)}
						</strong>
						<span className={badge(o.status)}>{o.status}</span>
					</div>
					<p>
						{o.base_asset} for {o.quote_asset}
					</p>
					<code>{o.id}</code>
					<small className="muted">{relativeTime(o.created_at)}</small>
					{o.expires_at && <small className="muted">Expires: {relativeTime(o.expires_at)}</small>}
				</article>
			))}
		</div>
	);
}

/* ─── Create Offer ─── */
function CreateOffer({ address }: { address: string }) {
	const [side, setSide] = useState("sell");
	const [baseAsset, setBaseAsset] = useState("KAS");
	const [quoteAsset, setQuoteAsset] = useState("USDC");
	const [amount, setAmount] = useState("");
	const [expireHours, setExpireHours] = useState("72");
	const [priceType, setPriceType] = useState("fixed");
	const [priceOffset, setPriceOffset] = useState("0");
	const [minPrice, setMinPrice] = useState("");
	const [maxPrice, setMaxPrice] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done">("idle");
	const { notify } = useToast();

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const amountNum = Number.parseFloat(amount);
		if (!amountNum || amountNum <= 0) return;
		setStatus("loading");
		try {
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
			});
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
					<option value="KRC20:NACHO">KRC20:NACHO</option>
					<option value="KRC20:KASPY">KRC20:KASPY</option>
				</select>
			</FormField>
			<FormField label="For asset">
				<select value={quoteAsset} onChange={(e) => setQuoteAsset(e.target.value)}>
					<option value="USDC">USDC</option>
					<option value="KAS">KAS</option>
					<option value="KRC20:NACHO">KRC20:NACHO</option>
					<option value="KRC20:KASPY">KRC20:KASPY</option>
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
