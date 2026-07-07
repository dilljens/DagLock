import { useState, useEffect } from "react";
import { api, type AuthHeaders } from "../api";
import { useWallet, useAddress } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { FormField } from "../ui";
import { Helmet } from "react-helmet-async";
import { EmptyState } from "../components/empty-state";
import { PriceAlertsSettings } from "../components/PriceAlerts";

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
		sign(msg).then((signature) => {
			const auth: AuthHeaders = { address, signature, message: msg };
			api.getNotifications(auth).then((d: any) => {
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
			}).catch(() => {});
		}).catch(() => {});
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
				<Helmet><title>Settings — DagLock</title></Helmet>
				<EmptyState icon="⚙️" title="Connect your wallet" description="Connect your wallet to manage notification settings." />
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
							<button className="button primary" onClick={handleSubscribe} disabled={loading === "subscribe"}>
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
							<label key={ev} style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "8px", cursor: "pointer" }}>
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
						<button className="button" onClick={handlePrefs} disabled={loading === "prefs"} style={{ marginTop: "8px" }}>
							{loading === "prefs" ? "Saving..." : "Save Preferences"}
						</button>
					</div>
				)}

				{/* Price alerts */}
				<PriceAlertsSettings address={address!} />
			</div>
		</>
	);
}
