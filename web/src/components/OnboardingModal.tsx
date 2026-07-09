import { useState, useEffect, useCallback } from "react";
import { useWallet } from "../context/WalletContext";
import { useRouter } from "../router";

const STORAGE_KEY = "daglock_onboarded";

const slides = [
	{
		title: "Welcome to DagLock",
		subtitle: "Trustless Escrow on Kaspa",
		body: "DagLock lets you trade KAS and KRC-20 tokens with anyone, anywhere — without trusting them. A SilverScript covenant holds the funds and enforces the rules. No admin keys, no backdoors, no banks.",
		icon: "🤝",
	},
	{
		title: "How It Works",
		subtitle: "4 simple steps",
		body: [
			"1. Propose — Create an escrow or accept an offer",
			"2. Lock — Buyer sends KAS to a covenant address (enforced by code)",
			"3. Confirm — Both parties agree the terms are met",
			"4. Settle — Funds release to seller. Or refund to buyer after timeout.",
		].join("\n"),
		icon: "🔄",
	},
	{
		title: "Get Started",
		subtitle: "Connect or try testnet",
		body: "Connect KasWare browser extension to get started. No KasWare? Use manual mode with any Kaspa wallet, or try the testnet guide to explore DagLock without connecting a wallet at all.",
		icon: "🚀",
	},
];

export function OnboardingModal() {
	const [visible, setVisible] = useState(false);
	const [slide, setSlide] = useState(0);
	const { state, connect } = useWallet();
	const { navigate } = useRouter();

	// Show modal on first visit (before wallet is connected)
	useEffect(() => {
		const dismissed = localStorage.getItem(STORAGE_KEY);
		if (!dismissed && !state.connected) {
			setVisible(true);
		}
	}, [state.connected]);

	// Keyboard navigation
	useEffect(() => {
		if (!visible) return;
		function onKey(e: KeyboardEvent) {
			if (e.key === "ArrowRight" || e.key === " ") {
				e.preventDefault();
				next();
			} else if (e.key === "ArrowLeft") {
				e.preventDefault();
				if (slide > 0) setSlide((s) => s - 1);
			} else if (e.key === "Escape") {
				dismiss();
			}
		}
		window.addEventListener("keydown", onKey);
		return () => window.removeEventListener("keydown", onKey);
	}, [visible, slide]);

	const dismiss = useCallback(() => {
		localStorage.setItem(STORAGE_KEY, "1");
		setVisible(false);
	}, []);

	function next() {
		if (slide < slides.length - 1) {
			setSlide(slide + 1);
		} else {
			dismiss();
		}
	}

	if (!visible) return null;

	const s = slides[slide];

	return (
		<div className="onboarding-overlay" onClick={dismiss}>
			<div className="onboarding-modal" onClick={(e) => e.stopPropagation()}>
				<button type="button" className="onboarding-skip" onClick={dismiss} aria-label="Skip">
					Skip tour
				</button>

				<div className="onboarding-icon">{s.icon}</div>
				<h2 className="onboarding-title">{s.title}</h2>
				<p className="onboarding-subtitle">{s.subtitle}</p>
				<p className="onboarding-body">{s.body}</p>

				<div className="onboarding-dots">
					{slides.map((_, i) => (
						<span
							key={i}
							className={`onboarding-dot ${i === slide ? "onboarding-dot--active" : ""}`}
						/>
					))}
				</div>

				<div className="onboarding-actions">
					{slide > 0 && (
						<button type="button" className="button" onClick={() => setSlide((s) => s - 1)}>
							Back
						</button>
					)}
					{slide === 0 && <div />}
					{slide < slides.length - 1 ? (
						<button type="button" className="button primary" onClick={next}>
							Next
						</button>
					) : (
						<div style={{ display: "flex", gap: "8px", width: "100%" }}>
							<button
								type="button"
								className="button"
								onClick={() => {
									dismiss();
									navigate("/testnet");
								}}
								style={{ flex: 1 }}
							>
								Try Testnet
							</button>
							<button
								type="button"
								className="button primary"
								onClick={() => {
									dismiss();
									if (!state.connected) connect();
								}}
								style={{ flex: 1 }}
							>
								Connect Wallet
							</button>
						</div>
					)}
				</div>
			</div>
		</div>
	);
}
