import { useState } from "react";
import { api, type CreateOfferRequest, type Offer } from "../api";
import { money, sompi, relativeTime, badge } from "../helpers";
import type { LoadState } from "../helpers";
import { FormField, ValidatedInput, kvad, LookupResult } from "../ui";

/* ─── Create Offer Form ─── */
export function CreateOfferForm({ onDone }: { onDone: () => void }) {
	const [side, setSide] = useState("sell");
	const [baseAsset, setBaseAsset] = useState("KAS");
	const [quoteAsset, setQuoteAsset] = useState("USDC");
	const [amount, setAmount] = useState("");
	const [address, setAddress] = useState("");
	const [counterparty, setCounterparty] = useState("");
	const [expireHours, setExpireHours] = useState("72");
	const [priceType, setPriceType] = useState("fixed");
	const [priceOffset, setPriceOffset] = useState("0");
	const [minPrice, setMinPrice] = useState("");
	const [maxPrice, setMaxPrice] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">("idle");
	const [error, setError] = useState("");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const amountNum = Number.parseFloat(amount);
		if (!amountNum || amountNum <= 0) {
			setError("Invalid amount. Please enter a positive number.");
			return;
		}
		const trimmedAddr = address.trim();
		if (!trimmedAddr.startsWith("kaspa:")) {
			setError(
				"Invalid address format. Must be a valid Kaspa address starting with 'kaspa:'. Check for leading/trailing spaces.",
			);
			return;
		}
		setStatus("loading");
		setError("");
		try {
			const body: CreateOfferRequest = {
				creator_address: trimmedAddr,
				side,
				base_asset: baseAsset,
				quote_asset: quoteAsset,
				amount_sompi: sompi(amountNum),
				expires_at: Math.floor(Date.now() / 1000) + (Number.parseInt(expireHours) || 72) * 3600,
				price_type: priceType,
			};
			if (priceType === "market") {
				body.price_offset = Number.parseFloat(priceOffset) || 0;
				if (minPrice) body.min_price = Number.parseFloat(minPrice);
				if (maxPrice) body.max_price = Number.parseFloat(maxPrice);
			}
			if (counterparty.startsWith("kaspa:")) body.counterparty_address = counterparty;
			await api.createOffer(body);
			setStatus("done");
			onDone();
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	if (status === "done") return <p className="muted success-text">Offer created!</p>;

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
					<option value="other">Other...</option>
				</select>
			</FormField>
			<FormField label="For asset">
				<select value={quoteAsset} onChange={(e) => setQuoteAsset(e.target.value)}>
					<option value="USDC">USDC</option>
					<option value="KAS">KAS</option>
					<option value="KRC20:NACHO">KRC20:NACHO</option>
					<option value="KRC20:KASPY">KRC20:KASPY</option>
					<option value="other">Other...</option>
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
			{(() => {
				const n = Number.parseFloat(amount) || 0;
				const fee = n * 0.005;
				if (n <= 0) return null;
				return (
					<p className="muted" style={{ fontSize: "13px", marginTop: "-8px" }}>
						Fee: {fee.toFixed(4)} KAS (0.5%)
						{n < 1 && (
							<span style={{ color: "#ff9800", marginLeft: "8px" }}>
								⚠️ Low amount — fee may be significant
							</span>
						)}
					</p>
				);
			})()}
			<ValidatedInput
				label="Your address"
				value={address}
				onChange={setAddress}
				placeholder="kaspa:..."
				validate={kvad}
			/>
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
			<FormField label="Counterparty (optional)">
				<input
					value={counterparty}
					onChange={(e) => setCounterparty(e.target.value)}
					placeholder="kaspa:..."
				/>
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Creating…" : "Create offer"}
			</button>
		</form>
	);
}

/* ─── Offer item with accept/cancel inline ─── */
export function OfferCard({
	offer,
	onMutated,
}: {
	offer: Offer;
	onMutated: () => void;
}) {
	const [status, setStatus] = useState<"idle" | "loading">("idle");
	const [error, setError] = useState("");
	const [counterparty, setCounterparty] = useState("");

	async function handleAccept() {
		if (!counterparty.startsWith("kaspa:")) return;
		setStatus("loading");
		setError("");
		try {
			await api.acceptOffer(offer.id, counterparty);
			onMutated();
		} catch (err) {
			setError((err as Error).message);
			setStatus("idle");
		}
	}

	async function handleCancel() {
		setStatus("loading");
		setError("");
		try {
			await api.cancelOffer(offer.id);
			onMutated();
		} catch (err) {
			setError((err as Error).message);
			setStatus("idle");
		}
	}

	const canAct = offer.status === "proposed";

	return (
		<article className="offer">
			<div className="offer-top">
				<strong>
					{offer.side.toUpperCase()} {money(offer.amount_sompi)}
				</strong>
				<span className={badge(offer.status)}>{offer.status}</span>
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
						placeholder="your kaspa address"
						className="offer-input"
					/>
					<button className="button primary" disabled={status === "loading"} onClick={handleAccept}>
						Accept
					</button>
					<button className="button" disabled={status === "loading"} onClick={handleCancel}>
						Cancel
					</button>
				</div>
			)}
			{error && <p className="muted error-text">{error}</p>}
		</article>
	);
}

/* ─── My Offers Panel ─── */
export function MyOffersPanel() {
	const [address, setAddress] = useState("");
	const [filter, setFilter] = useState("all");
	const [list, setList] = useState<LoadState<Offer[]>>({ loading: false });

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		if (!address.trim()) return;
		setList({ loading: true });
		try {
			const data = await api.offers(address.trim());
			setList({ data: data.offers, loading: false });
		} catch (err) {
			setList({ error: (err as Error).message, loading: false });
		}
	}

	const filtered = list.data?.filter((o) => {
		if (filter === "all") return true;
		return o.status === filter;
	});

	return (
		<div className="stack">
			<form className="form" onSubmit={handleSubmit}>
				<input
					value={address}
					onChange={(e) => setAddress(e.target.value)}
					placeholder="your kaspa address"
				/>
				<button className="button" type="submit">
					List my offers
				</button>
			</form>
			{list.data && list.data.length > 0 && (
				<div className="action-tabs" style={{ marginTop: "8px" }}>
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
			<LookupResult
				loading={list.loading}
				error={list.error}
				data={filtered}
				render={(data) => (
					<div>
						{data.length === 0 && (
							<p className="muted">
								No {filter === "all" ? "" : filter} offers found for this address.
							</p>
						)}
						{data.map((o) => (
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
								{o.expires_at && (
									<small className="muted">Expires: {relativeTime(o.expires_at)}</small>
								)}
							</article>
						))}
					</div>
				)}
			/>
		</div>
	);
}
