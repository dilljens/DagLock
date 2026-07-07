import { useState, useEffect } from "react";
import { api } from "../api";
import { useRouter } from "../router";
import { relativeTime } from "../helpers";
import type { LoadState } from "../helpers";
import { Helmet } from "react-helmet-async";
import { SkeletonTable } from "../ui";

type TokenSummary = {
	ticker: string;
	price_kas: number | null;
	volume_24h_sompi: number;
	trades_24h: number;
	total_trades: number;
	active_offers: number;
	last_trade_at: number | null;
};

type TokenListResponse = {
	tokens: TokenSummary[];
	total: number;
};

function formatKasPrice(price: number | null): string {
	if (price === null || price === 0) return "—";
	if (price < 0.0001) return price.toExponential(2);
	if (price < 1) return price.toFixed(6);
	return price.toFixed(2);
}

function formatVolume(sompi: number): string {
	const kas = sompi / 100_000_000;
	if (kas >= 1_000_000) return `${(kas / 1_000_000).toFixed(1)}M KAS`;
	if (kas >= 1_000) return `${(kas / 1_000).toFixed(1)}K KAS`;
	if (kas >= 1) return `${kas.toFixed(2)} KAS`;
	return `${kas.toFixed(4)} KAS`;
}

export function TokensPage() {
	const { navigate } = useRouter();
	const [data, setData] = useState<LoadState<TokenSummary[]>>({ loading: true });
	const [search, setSearch] = useState("");
	const [sortBy, setSortBy] = useState<"volume" | "trades" | "price" | "name">("volume");

	useEffect(() => {
		api
			.tokens()
			.then((d: TokenListResponse) => setData({ data: d.tokens, loading: false }))
			.catch((e: Error) => setData({ error: e.message, loading: false }));
	}, []);

	const sorted = (data.data || [])
		.filter((t) => t.ticker.toLowerCase().includes(search.toLowerCase()))
		.sort((a, b) => {
			switch (sortBy) {
				case "volume": return b.volume_24h_sompi - a.volume_24h_sompi;
				case "trades": return b.trades_24h - a.trades_24h;
				case "price": return (b.price_kas || 0) - (a.price_kas || 0);
				case "name": return a.ticker.localeCompare(b.ticker);
				default: return 0;
			}
		});

	return (
		<>
			<Helmet>
				<title>KRC-20 Tokens — DagLock</title>
				<meta
					name="description"
					content="Browse KRC-20 tokens traded on DagLock escrow. Price, volume, and trade data from on-chain activity."
				/>
				<link rel="canonical" href="https://daglock.com/tokens" />
			</Helmet>
			<div>
				<div className="page-header">
					<h1>KRC-20 Tokens</h1>
					<p>Browse tokens traded on DagLock escrow. Data from on-chain offers and settlements.</p>
				</div>

				{/* Search + sort bar */}
				<div style={{ display: "flex", gap: "12px", marginBottom: "16px", alignItems: "center" }}>
					<input
						value={search}
						onChange={(e) => setSearch(e.target.value)}
						placeholder="Search by ticker…"
						style={{ flex: 1 }}
					/>
					<select
						value={sortBy}
						onChange={(e) => setSortBy(e.target.value as typeof sortBy)}
						style={{ width: "auto" }}
					>
						<option value="volume">By Volume</option>
						<option value="trades">By Trades</option>
						<option value="price">By Price</option>
						<option value="name">By Name</option>
					</select>
				</div>

				{data.loading && <SkeletonTable rows={5} />}
				{data.error && <p className="muted error-text">{data.error}</p>}

				{!data.loading && sorted.length === 0 && (
					<p className="muted" style={{ textAlign: "center", padding: "32px" }}>
						{search ? "No tokens match your search." : "No KRC-20 tokens found. Be the first to create an offer!"}
					</p>
				)}

				{/* Token cards/table */}
				<div className="token-grid">
					{sorted.map((t) => (
						<article
							key={t.ticker}
							className="token-card"
							onClick={() => navigate(`/tokens/${t.ticker}` as any)}
							style={{ cursor: "pointer" }}
						>
							<div className="token-card-header">
								<span className="token-card-ticker">{t.ticker}</span>
								<span className="token-card-price">
									{formatKasPrice(t.price_kas)} KAS
								</span>
							</div>
							<div className="token-card-stats">
								<div className="token-card-stat">
									<span className="token-card-stat-label">Volume 24h</span>
									<span className="token-card-stat-value">{formatVolume(t.volume_24h_sompi)}</span>
								</div>
								<div className="token-card-stat">
									<span className="token-card-stat-label">Trades</span>
									<span className="token-card-stat-value">{t.trades_24h}</span>
								</div>
								<div className="token-card-stat">
									<span className="token-card-stat-label">Offers</span>
									<span className="token-card-stat-value">{t.active_offers}</span>
								</div>
							</div>
							{t.last_trade_at && (
								<div className="token-card-footer">
									<span className="muted">Last trade: {relativeTime(t.last_trade_at)}</span>
								</div>
							)}
						</article>
					))}
				</div>
			</div>
		</>
	);
}
