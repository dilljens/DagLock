import { useState } from "react";
import { api, type AuthHeaders } from "../api";
import { useWallet, useAddress } from "../context/WalletContext";
import { useRouter } from "../router";
import { useToast } from "../layout/Toast";
import { FormField } from "../ui";
import { Helmet } from "react-helmet-async";
import { EmptyState } from "../components/empty-state";

type Tab = "create" | "manage";

const TEMPLATES = [
	{
		name: "Standard KRC-20",
		desc: "Fixed supply, 1B tokens, 8 decimals",
		supply: 1_000_000_000,
		decimals: 8,
		mode: "fixed",
	},
	{
		name: "Community Token",
		desc: "Mintable, 100M supply, 8 decimals",
		supply: 100_000_000,
		decimals: 8,
		mode: "mintable",
	},
	{
		name: "Memecoin",
		desc: "Fixed supply, 1T tokens, 8 decimals",
		supply: 1_000_000_000_000,
		decimals: 8,
		mode: "fixed",
	},
];

export function CreateTokenPage() {
	const { navigate } = useRouter();
	const address = useAddress();
	const { state: wallet, sign } = useWallet();
	const { notify } = useToast();

	const [step, setStep] = useState(1);
	const [name, setName] = useState("");
	const [ticker, setTicker] = useState("");
	const [supply, setSupply] = useState("1000000000");
	const [decimals, setDecimals] = useState("8");
	const [mintMode, setMintMode] = useState("fixed");
	const [errors, setErrors] = useState<Record<string, string>>({});
	const [loading, setLoading] = useState(false);
	const [result, setResult] = useState<any>(null);

	// Quick template selection
	function applyTemplate(t: typeof TEMPLATES[0]) {
		setSupply(t.supply.toString());
		setDecimals(t.decimals.toString());
		setMintMode(t.mode);
	}

	function validate(): boolean {
		const e: Record<string, string> = {};
		if (name.trim().length < 2 || name.trim().length > 64)
			e.name = "Name must be 2-64 characters";
		const t = ticker.trim().toUpperCase();
		if (t.length < 3 || t.length > 8) e.ticker = "Ticker must be 3-8 characters";
		if (!/^[A-Z0-9]+$/.test(t)) e.ticker = "Ticker must be alphanumeric";
		const s = BigInt(supply || "0");
		if (s <= 0n || s > 1_000_000_000_000n) e.supply = "Supply must be between 1 and 1,000,000,000,000";
		const d = parseInt(decimals);
		if (isNaN(d) || d < 0 || d > 18) e.decimals = "Decimals must be 0-18";
		if (!address) e.address = "Connect your wallet first";
		setErrors(e);
		return Object.keys(e).length === 0;
	}

	async function handleDeploy() {
		if (!validate()) return;
		if (!address) return;

		setLoading(true);
		try {
			const message = `deploy_token:${ticker.trim().toUpperCase()}:${Math.floor(Date.now() / 1000)}`;
			const signature = await sign(message);

			const auth: AuthHeaders = { address, signature, message };
			const result = await api.deployToken(
				{
					name: name.trim(),
					ticker: ticker.trim(),
					total_supply: Number(supply),
					decimals: parseInt(decimals),
					mint_mode: mintMode,
					owner_address: address,
				},
				auth,
			);
			setResult(result);
			setStep(4);
			notify("success", "Token registered!");
		} catch (err) {
			notify("error", "Deployment failed", (err as Error).message);
		} finally {
			setLoading(false);
		}
	}

	if (!wallet.connected) {
		return (
			<>
				<Helmet><title>Create Token — DagLock</title></Helmet>
				<EmptyState
					icon="🏷️"
					title="Connect your wallet"
					description="Connect KasWare to create a KRC-20 token."
				/>
			</>
		);
	}

	return (
		<>
			<Helmet>
				<title>Create Token — DagLock</title>
				<meta name="description" content="Create a KRC-20 token on Kaspa in a few clicks. Launch your token, bootstrap liquidity with DagLock escrow." />
			</Helmet>
			<div>
				<div className="page-header">
					<h1>Create Token</h1>
					<p>Launch a KRC-20 token on Kaspa. Register here, then broadcast the covenant.</p>
				</div>

				<div className="tab-bar">
					<button className={`tab-btn ${step === 1 ? "tab-btn--active" : ""}`} onClick={() => { setStep(1); setResult(null); }}>
						Details
					</button>
					<button className={`tab-btn ${step === 2 ? "tab-btn--active" : ""}`} disabled>
						Review
					</button>
					<button className={`tab-btn ${step === 3 ? "tab-btn--active" : ""}`} disabled>
						Sign
					</button>
					<button className={`tab-btn ${step === 4 ? "tab-btn--active" : ""}`} disabled>
						Done
					</button>
				</div>

				{/* Step 1: Token Details */}
				{step === 1 && (
					<div>
						<div className="panel" style={{ marginBottom: "16px" }}>
							<h3 style={{ margin: "0 0 8px" }}>Token Templates</h3>
							<p className="muted" style={{ margin: "0 0 12px", fontSize: "13px" }}>
								Quick-start presets — you can customize everything below.
							</p>
							<div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
								{TEMPLATES.map((t) => (
									<button
										key={t.name}
										className="button"
										onClick={() => applyTemplate(t)}
										style={{ fontSize: "12px", padding: "4px 12px" }}
									>
										{t.name}
									</button>
								))}
							</div>
						</div>

						<form className="form form-stacked" onSubmit={(e) => { e.preventDefault(); if (validate()) setStep(2); }}>
							<FormField label="Token name">
								<input value={name} onChange={(e) => setName(e.target.value)} placeholder="My Kaspa Token" maxLength={64} />
								{errors.name && <span className="muted error-text" style={{ fontSize: "12px" }}>{errors.name}</span>}
							</FormField>
							<FormField label="Ticker">
								<input value={ticker} onChange={(e) => setTicker(e.target.value.toUpperCase())} placeholder="MKT" maxLength={8} style={{ textTransform: "uppercase" }} />
								{errors.ticker && <span className="muted error-text" style={{ fontSize: "12px" }}>{errors.ticker}</span>}
							</FormField>
							<FormField label="Total supply">
								<input type="number" value={supply} onChange={(e) => setSupply(e.target.value)} min={1} />
								{errors.supply && <span className="muted error-text" style={{ fontSize: "12px" }}>{errors.supply}</span>}
							</FormField>
							<FormField label="Decimals">
								<input type="number" value={decimals} onChange={(e) => setDecimals(e.target.value)} min={0} max={18} />
								{errors.decimals && <span className="muted error-text" style={{ fontSize: "12px" }}>{errors.decimals}</span>}
							</FormField>
							<FormField label="Mint mode">
								<select value={mintMode} onChange={(e) => setMintMode(e.target.value)}>
									<option value="fixed">Fixed supply (all minted at deploy)</option>
									<option value="mintable">Mintable (owner can mint more)</option>
									<option value="burnable">Burnable (holders can burn)</option>
								</select>
							</FormField>
							<button className="button primary" type="submit" style={{ marginTop: "12px" }}>
								Next: Review
							</button>
						</form>
					</div>
				)}

				{/* Step 2: Review */}
				{step === 2 && (
					<div>
						<div className="panel" style={{ marginBottom: "16px" }}>
							<h3 style={{ margin: "0 0 12px" }}>Review Token</h3>
							<div className="stack">
								<div className="row"><span>Name</span><strong>{name}</strong></div>
								<div className="row"><span>Ticker</span><strong>{ticker.toUpperCase()}</strong></div>
								<div className="row"><span>Supply</span><strong>{BigInt(supply).toLocaleString()}</strong></div>
								<div className="row"><span>Decimals</span><strong>{decimals}</strong></div>
								<div className="row"><span>Mint mode</span><strong>{mintMode}</strong></div>
								<div className="row"><span>Owner</span><code>{address?.slice(0, 24)}...</code></div>
							</div>
						</div>

						<p className="muted" style={{ fontSize: "13px" }}>
							The KRC-20 covenant needs to be compiled and broadcast separately. After
							registering, you'll get instructions for the on-chain deployment.
						</p>

						<div style={{ display: "flex", gap: "8px" }}>
							<button className="button" onClick={() => setStep(1)}>Back</button>
							<button className="button primary" onClick={() => setStep(3)} disabled={loading}>
								Next: Sign
							</button>
						</div>
					</div>
				)}

				{/* Step 3: Sign & Deploy */}
				{step === 3 && (
					<div>
						<div className="panel" style={{ marginBottom: "16px" }}>
							<h3 style={{ margin: "0 0 8px" }}>Sign & Register</h3>
							<p className="muted" style={{ fontSize: "13px" }}>
								Sign a message with your wallet to prove ownership of this address. This
								registers the token in the DagLock indexer — no on-chain broadcast yet.
							</p>
						</div>

						<button className="button primary" onClick={handleDeploy} disabled={loading}>
							{loading ? "Registering..." : "Sign & Register Token"}
						</button>
					</div>
				)}

				{/* Step 4: Done */}
				{step === 4 && result && (
					<div style={{ textAlign: "center", padding: "24px 0" }}>
						<div style={{ fontSize: "48px", marginBottom: "16px" }}>✅</div>
						<h3 style={{ margin: "0 0 8px" }}>Token Registered!</h3>
						<p className="muted" style={{ margin: "0 0 20px" }}>
							{ticker.toUpperCase()} is now registered in the DagLock indexer.
						</p>

						<div className="panel" style={{ textAlign: "left", marginBottom: "20px" }}>
							<h4 style={{ margin: "0 0 8px" }}>Next Steps</h4>
							<ol style={{ lineHeight: 2, fontSize: "14px" }}>
								<li>Compile the KRC-20 covenant using the compiler API</li>
								<li>Broadcast the covenant transaction from your wallet</li>
								<li>
									Update the deployment status:{" "}
									<code style={{ fontSize: "12px" }}>
										PATCH /v1/tokens/{ticker.toUpperCase()}
									</code>
								</li>
								<li>
									<button
										className="button"
										onClick={() => navigate("/offers" as any)}
										style={{ fontSize: "12px", padding: "2px 10px", marginTop: "4px" }}
									>
										Create a buy offer for {ticker.toUpperCase()}
									</button>
								</li>
							</ol>
						</div>

						<button className="button primary" onClick={() => navigate(`/tokens/${ticker.toUpperCase()}` as any)}>
							View Token Page
						</button>
					</div>
				)}
			</div>
		</>
	);
}
