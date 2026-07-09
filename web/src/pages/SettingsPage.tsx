import { useState, useEffect } from "react";
import { api, type AccountFlags, type ApiKey, type App, type AuthHeaders } from "../api";
import { useWallet, useAddress } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { FormField } from "../ui";
import { Helmet } from "react-helmet-async";
import { EmptyState } from "../components/empty-state";
import { PriceAlertsSettings } from "../components/PriceAlerts";
import { mockSignature } from "../kasware";

export function SettingsPage() {
	const { state: wallet, sign } = useWallet();
	const address = useAddress();
	const { notify } = useToast();

	const [email, setEmail] = useState("");
	const [verified, setVerified] = useState(false);
	const [subscribed, setSubscribed] = useState(false);
	const [verifyCode, setVerifyCode] = useState("");
	const [showVerify, setShowVerify] = useState(false);
	const [prefs, setPrefs] = useState({
		notify_created: true,
		notify_settled: true,
		notify_disputed: true,
		notify_refunded: true,
		notify_expired: true,
	});
	const [loading, setLoading] = useState("");

	useEffect(() => {
		if (!address) return;
		const msg = `get_notifications:${Math.floor(Date.now() / 1000)}`;
		sign(msg)
			.then((signature) => {
				const auth: AuthHeaders = { address, signature, message: msg };
				api
					.getNotifications(auth)
					.then((d: any) => {
						if (d.email) {
							setEmail(d.email);
							setSubscribed(true);
							setVerified(d.email_verified);
							setPrefs({
								notify_created: d.notify_created ?? true,
								notify_settled: d.notify_settled ?? true,
								notify_disputed: d.notify_disputed ?? true,
								notify_refunded: d.notify_refunded ?? true,
								notify_expired: d.notify_expired ?? true,
							});
						}
					})
					.catch(() => {});
			})
			.catch(() => {});
	}, [address]);

	async function handleSubscribe() {
		if (!address || !email.includes("@")) return;
		setLoading("subscribe");
		try {
			const msg = `subscribe:${email}:${Math.floor(Date.now() / 1000)}`;
			const signature = await sign(msg);
			const auth: AuthHeaders = { address, signature, message: msg };
			await api.subscribeNotifications({ email }, auth);
			setSubscribed(true);
			setShowVerify(true);
			notify("success", "Verification code sent to your email!");
		} catch (err) {
			notify("error", "Failed to subscribe", (err as Error).message);
		} finally {
			setLoading("");
		}
	}

	async function handleVerify() {
		if (!address || !verifyCode) return;
		setLoading("verify");
		try {
			const msg = `verify:${verifyCode}:${Math.floor(Date.now() / 1000)}`;
			const signature = await sign(msg);
			const auth: AuthHeaders = { address, signature, message: msg };
			await api.verifyNotifications({ code: verifyCode }, auth);
			setVerified(true);
			setShowVerify(false);
			notify("success", "Email verified!");
		} catch (err) {
			notify("error", "Verification failed", (err as Error).message);
		} finally {
			setLoading("");
		}
	}

	async function handlePrefs() {
		if (!address) return;
		setLoading("prefs");
		try {
			const msg = `prefs:${Math.floor(Date.now() / 1000)}`;
			const signature = await sign(msg);
			const auth: AuthHeaders = { address, signature, message: msg };
			await api.updateNotificationPrefs(prefs, auth);
			notify("success", "Preferences saved!");
		} catch (err) {
			notify("error", "Failed to save", (err as Error).message);
		} finally {
			setLoading("");
		}
	}

	if (!wallet.connected) {
		return (
			<>
				<Helmet>
					<title>Settings — DagLock</title>
				</Helmet>
				<EmptyState
					icon="⚙️"
					title="Connect your wallet"
					description="Connect your wallet to manage notification settings."
				/>
			</>
		);
	}

	return (
		<>
			<Helmet>
				<title>Settings — DagLock</title>
				<meta name="description" content="Manage your DagLock notification preferences." />
			</Helmet>
			<div>
				<div className="page-header">
					<h1>Settings</h1>
					<p>Manage email notification preferences for escrow events.</p>
				</div>

				{/* Email subscription */}
				<div className="panel" style={{ marginBottom: "16px" }}>
					<h3 style={{ margin: "0 0 4px" }}>Email Notifications</h3>
					<p className="muted" style={{ margin: "0 0 12px", fontSize: "13px" }}>
						Receive email alerts when your escrows change status.
					</p>

					{subscribed ? (
						<div>
							<p style={{ fontSize: "14px", marginBottom: "8px" }}>
								📧 {email} {verified ? "✅ Verified" : "❌ Not verified"}
							</p>
							{!verified && (
								<div style={{ marginBottom: "12px" }}>
									<input
										value={verifyCode}
										onChange={(e) => setVerifyCode(e.target.value.toUpperCase())}
										placeholder="Enter verification code"
										maxLength={6}
										style={{ width: "200px", marginRight: "8px" }}
									/>
									<button className="button" onClick={handleVerify} disabled={loading === "verify"}>
										{loading === "verify" ? "Verifying..." : "Verify"}
									</button>
								</div>
							)}
						</div>
					) : (
						<div style={{ display: "flex", gap: "8px", marginBottom: "12px" }}>
							<input
								value={email}
								onChange={(e) => setEmail(e.target.value)}
								placeholder="your@email.com"
								style={{ flex: 1 }}
							/>
							<button
								className="button primary"
								onClick={handleSubscribe}
								disabled={loading === "subscribe"}
							>
								{loading === "subscribe" ? "Subscribing..." : "Subscribe"}
							</button>
						</div>
					)}
				</div>

				{/* Notification preferences */}
				{subscribed && verified && (
					<div className="panel" style={{ marginBottom: "16px" }}>
						<h3 style={{ margin: "0 0 12px" }}>Notification Preferences</h3>
						{(["created", "settled", "disputed", "refunded", "expired"] as const).map((ev) => (
							<label
								key={ev}
								style={{
									display: "flex",
									alignItems: "center",
									gap: "8px",
									marginBottom: "8px",
									cursor: "pointer",
								}}
							>
								<input
									type="checkbox"
									checked={prefs[`notify_${ev}` as keyof typeof prefs]}
									onChange={(e) => setPrefs({ ...prefs, [`notify_${ev}`]: e.target.checked })}
								/>
								<span style={{ fontSize: "14px" }}>
									Escrow {ev.charAt(0).toUpperCase() + ev.slice(1)}
								</span>
							</label>
						))}
						<button
							className="button"
							onClick={handlePrefs}
							disabled={loading === "prefs"}
							style={{ marginTop: "8px" }}
						>
							{loading === "prefs" ? "Saving..." : "Save Preferences"}
						</button>
					</div>
				)}

				{/* Price alerts */}
				<PriceAlertsSettings address={address!} />

				{/* API Key Management */}
				<ApiKeySection address={address!} sign={sign} notify={notify} />

				{/* Bot registration */}
				<BotRegistrationSection address={address!} sign={sign} notify={notify} />
			</div>
		</>
	);
}

/* ─── API Key Management ─── */
function ApiKeySection({
	address,
	notify,
}: {
	address: string;
	sign: (msg: string) => Promise<string>;
	notify: (type: "success" | "error" | "info", title: string, message?: string) => void;
}) {
	const [app, setApp] = useState<App | null>(null);
	const [keys, setKeys] = useState<ApiKey[]>([]);
	const [apiKey, setApiKey] = useState<string | null>(null);
	const [loading, setLoading] = useState(false);
	const [showNewKey, setShowNewKey] = useState<string | null>(null);
	const storedKey = localStorage.getItem("daglock_api_key");

	// On mount: try to load app using stored API key
	useEffect(() => {
		if (!storedKey) return;
		// The stored key is "app_id:api_key_plaintext"
		const [appId] = storedKey.split(":");
		if (!appId) return;
		api
			.getApp(appId, storedKey)
			.then((a) => {
				setApp(a);
				setApiKey(storedKey);
				// Load keys
				api.listApiKeys(appId, storedKey).then((k) => setKeys(k.keys)).catch(() => {});
			})
			.catch(() => {
				// Stored key is invalid — clear it
				localStorage.removeItem("daglock_api_key");
			});
	}, []);

	async function handleRegister() {
		setLoading(true);
		try {
			const result = await api.registerApp({
				name: `${address.slice(0, 10)}… App`,
				owner_address: address,
			});
			const stored = `${result.app.id}:${result.api_key}`;
			localStorage.setItem("daglock_api_key", stored);
			setApp(result.app);
			setApiKey(stored);
			setShowNewKey(result.api_key);
			setKeys([]);
			notify("success", "App registered!");
		} catch (err) {
			notify("error", "Registration failed", (err as Error).message);
		} finally {
			setLoading(false);
		}
	}

	async function handleCreateKey() {
		if (!app || !apiKey) return;
		setLoading(true);
		try {
			const result = await api.createApiKey(app.id, apiKey);
			setShowNewKey(result.api_key);
			// Reload keys
			const k = await api.listApiKeys(app.id, apiKey);
			setKeys(k.keys);
			notify("success", "New API key created!");
		} catch (err) {
			notify("error", "Failed to create key", (err as Error).message);
		} finally {
			setLoading(false);
		}
	}

	async function handleDeleteKey(keyId: string) {
		if (!app || !apiKey) return;
		setLoading(true);
		try {
			await api.deleteApiKey(app.id, keyId, apiKey);
			setKeys((prev) => prev.filter((k) => k.key_id !== keyId));
			notify("success", "API key revoked");
		} catch (err) {
			notify("error", "Failed to revoke key", (err as Error).message);
		} finally {
			setLoading(false);
		}
	}

	function handleDisconnect() {
		localStorage.removeItem("daglock_api_key");
		setApp(null);
		setKeys([]);
		setApiKey(null);
		setShowNewKey(null);
	}

	return (
		<div className="panel" style={{ marginBottom: "16px" }}>
			<h3 style={{ margin: "0 0 4px" }}>🔑 API Keys</h3>
			<p className="muted" style={{ margin: "0 0 12px", fontSize: "13px" }}>
				API keys let automated trading bots and scripts interact with DagLock programmatically.
				Each key has a rate limit tier (Free: 10 req/min, Pro: 100, Whale: 1000).
			</p>

			{!app ? (
				<button
					className="button primary"
					onClick={handleRegister}
					disabled={loading}
				>
					{loading ? "Registering..." : "Register App & Get API Key"}
				</button>
			) : (
				<div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
					<div
						style={{
							display: "flex",
							justifyContent: "space-between",
							alignItems: "center",
						}}
					>
						<span style={{ fontSize: "13px" }}>
							App: <strong>{app.name}</strong>
							<span className="muted" style={{ marginLeft: "8px", fontSize: "11px" }}>
								(created {new Date(app.created_at * 1000).toLocaleDateString()})
							</span>
						</span>
						<button
							className="button"
							onClick={handleDisconnect}
							style={{ fontSize: "11px", padding: "2px 8px" }}
						>
							Disconnect
						</button>
					</div>

					{/* New key alert */}
					{showNewKey && (
						<div
							style={{
								background: "#1a3a1a",
								border: "1px solid var(--color-primary)",
								borderRadius: "8px",
								padding: "12px",
							}}
						>
							<p style={{ margin: "0 0 4px", fontSize: "13px", color: "var(--color-primary)" }}>
								⚠️ New API Key — save this now!
							</p>
							<p className="muted" style={{ margin: "0 0 4px", fontSize: "11px" }}>
								This key will only be shown once. Store it securely.
							</p>
							<code
								style={{
									display: "block",
									padding: "8px",
									background: "#0a1a0a",
									borderRadius: "4px",
									fontSize: "12px",
									wordBreak: "break-all",
								}}
							>
								{showNewKey}
							</code>
							<button
								className="button"
								onClick={() => navigator.clipboard.writeText(showNewKey)}
								style={{ fontSize: "11px", padding: "2px 8px", marginTop: "6px" }}
							>
								Copy to clipboard
							</button>
						</div>
					)}

					{/* Key list */}
					{keys.length > 0 && (
						<div>
							<span style={{ fontSize: "12px", color: "#888" }}>Active API keys:</span>
							{keys.map((k) => (
								<div
									key={k.key_id}
									style={{
										display: "flex",
										justifyContent: "space-between",
										alignItems: "center",
										padding: "6px 0",
										borderBottom: "1px solid var(--color-border)",
										fontSize: "12px",
									}}
								>
									<div>
										<span
											className="pill"
											style={{
												fontSize: "10px",
												background: k.tier === "whale" ? "#9c27b022" : k.tier === "pro" ? "#2196f322" : "#88888822",
												color: k.tier === "whale" ? "#ce93d8" : k.tier === "pro" ? "#64b5f6" : "#888",
												marginRight: "6px",
											}}
										>
											{k.tier}
										</span>
										<span style={{ color: "var(--color-text-secondary)" }}>
											{k.key_id.slice(0, 8)}…
										</span>
										{k.last_used_at && (
											<span className="muted" style={{ marginLeft: "6px", fontSize: "10px" }}>
												last used {new Date(k.last_used_at * 1000).toLocaleDateString()}
											</span>
										)}
									</div>
									<button
										className="button"
										onClick={() => handleDeleteKey(k.key_id)}
										disabled={loading}
										style={{ fontSize: "10px", padding: "2px 6px", color: "#ff7b7b" }}
									>
										Revoke
									</button>
								</div>
							))}
						</div>
					)}

					<button
						className="button"
						onClick={handleCreateKey}
						disabled={loading}
						style={{ alignSelf: "flex-start" }}
					>
						{loading ? "Creating..." : "Generate New Key"}
					</button>
				</div>
			)}
		</div>
	);
}

/* ─── Bot Registration ─── */
function BotRegistrationSection({
	address,
	sign,
	notify,
}: {
	address: string;
	sign: (msg: string) => Promise<string>;
	notify: (type: "success" | "error" | "info", title: string, message?: string) => void;
}) {
	const [flags, setFlags] = useState<AccountFlags | null>(null);
	const [loading, setLoading] = useState(false);
	const [isBot, setIsBot] = useState(false);
	const [label, setLabel] = useState("");

	// Load current flags
	useEffect(() => {
		api
			.getFlags(address)
			.then((f) => {
				setFlags(f);
				setIsBot(f.is_bot);
				setLabel(f.label || "");
			})
			.catch(() => {});
	}, [address]);

	async function handleSave() {
		setLoading(true);
		try {
			const result = await api.setFlags({ address, is_bot: isBot, label: label || null });
			setFlags({ address, is_bot: isBot, label: label || null, updated_at: Math.floor(Date.now() / 1000) });
			notify("success", isBot ? "Registered as bot" : "Bot flag removed");
		} catch (err) {
			notify("error", "Failed to update bot registration", (err as Error).message);
		} finally {
			setLoading(false);
		}
	}

	return (
		<div className="panel" style={{ marginBottom: "16px" }}>
			<h3 style={{ margin: "0 0 4px" }}>🤖 Bot Registration</h3>
			<p className="muted" style={{ margin: "0 0 12px", fontSize: "13px" }}>
				If this address runs an automated trading bot, register it here. Bot-made offers
				will show a <strong>🤖 Bot</strong> badge on the offer board so other users know
				they're trading with an automated system.
			</p>

			<div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
				<label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer" }}>
					<input
						type="checkbox"
						checked={isBot}
						onChange={(e) => setIsBot(e.target.checked)}
					/>
					<span style={{ fontSize: "14px" }}>
						This address is a trading bot
					</span>
				</label>

				<div>
					<label style={{ fontSize: "12px", color: "#888", display: "block", marginBottom: "4px" }}>
						Label (optional)
					</label>
					<input
						value={label}
						onChange={(e) => setLabel(e.target.value)}
						placeholder="e.g. KAS Market Maker v2"
						style={{ width: "100%", maxWidth: "400px" }}
						maxLength={100}
					/>
				</div>

				<button
					className="button primary"
					onClick={handleSave}
					disabled={loading}
					style={{ alignSelf: "flex-start" }}
				>
					{loading ? "Saving..." : "Save Bot Registration"}
				</button>

				{flags && (
					<p className="muted" style={{ fontSize: "11px", margin: 0 }}>
						Last updated: {new Date(flags.updated_at * 1000).toLocaleString()}
					</p>
				)}
			</div>
		</div>
	);
}
