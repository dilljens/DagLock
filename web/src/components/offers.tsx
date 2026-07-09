import { useState, useEffect } from "react";
import { api, type CreateOfferRequest, type Offer, type TokenRegistryEntry } from "../api";
import { sompi } from "../helpers";
import { FormField, ValidatedInput, kvad } from "../ui";

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
	const [registeredTokens, setRegisteredTokens] = useState<TokenRegistryEntry[]>([]);

	useEffect(() => {
		api.registeredTokens().then((d) => setRegisteredTokens(d.tokens)).catch(() => {});
	}, []);

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
			{(() => {
				const n = Number.parseFloat(amount) || 0;
				const fee = n * 0.005;
				if (n <= 0) return null;
				return (
					<p className="muted" style={{ fontSize: "13px", marginTop: "-8px" }}>
						Fee: {fee.toFixed(4)} KAS (0.5%)
						{n < 1 && (
							<span style={{ color: "#ff9800", marginLeft: "8px" }}>
								Low amount — fee may be significant
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
