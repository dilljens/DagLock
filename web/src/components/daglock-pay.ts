const KASWARE_URL = "https://kasware.com";
const API_BASE = (window as any).__DAGLOCK_API_BASE__ || "";
const DAGLOCK_URL = (window as any).__DAGLOCK_URL__ || "https://daglock.com";

interface PaymentState {
	amount: string;
	asset: string;
	seller: string;
	memo: string;
	apiKey: string;
	theme: "light" | "dark";
	label: string;
	step: "ready" | "connecting" | "funding" | "waiting" | "complete" | "error";
	escrowId: string | null;
	error: string | null;
}

const template = document.createElement("template");
template.innerHTML = `
<style>
	:host {
		--dl-primary: var(--daglock-primary, #53d769);
		--dl-bg: var(--daglock-bg, #1a1a2e);
		--dl-text: var(--daglock-text, #e0e0e0);
		--dl-border: var(--daglock-border, #333);
		--dl-error: var(--daglock-error, #ff4444);
		--dl-font: var(--daglock-font, system-ui, sans-serif);
		--dl-radius: var(--daglock-radius, 8px);
		display: inline-block;
	}
	button {
		font-family: var(--dl-font);
		font-size: 16px;
		font-weight: 600;
		padding: 12px 24px;
		border: none;
		border-radius: var(--dl-radius);
		cursor: pointer;
		transition: opacity 0.2s, transform 0.1s;
		min-width: 200px;
		text-align: center;
		line-height: 1.4;
	}
	button:hover:not(:disabled) { opacity: 0.9; transform: translateY(-1px); }
	button:disabled { opacity: 0.5; cursor: not-allowed; }
	button.primary {
		background: var(--dl-primary);
		color: #000;
	}
	button.secondary {
		background: transparent;
		color: var(--dl-text);
		border: 1px solid var(--dl-border);
	}
	button.outline {
		background: transparent;
		color: var(--dl-primary);
		border: 2px solid var(--dl-primary);
	}
	.status {
		margin-top: 8px;
		font-family: var(--dl-font);
		font-size: 13px;
		color: var(--dl-text);
		opacity: 0.8;
	}
	.status.error { color: var(--dl-error); opacity: 1; }
	.status.complete { color: var(--dl-primary); opacity: 1; }
	.spinner {
		display: inline-block;
		width: 14px;
		height: 14px;
		border: 2px solid var(--dl-border);
		border-top-color: var(--dl-primary);
		border-radius: 50%;
		animation: dl-spin 0.6s linear infinite;
		vertical-align: middle;
		margin-right: 6px;
	}
	@keyframes dl-spin { to { transform: rotate(360deg); } }
	.link {
		color: var(--dl-primary);
		text-decoration: underline;
		cursor: pointer;
		background: none;
		border: none;
		font-family: var(--dl-font);
		font-size: 13px;
		padding: 0;
	}
</style>
<button type="button" id="btn" class="primary" part="button">Pay</button>
<div id="status" class="status" part="status"></div>
`;

export class DaglockPay extends HTMLElement {
	private state: PaymentState;
	private btn: HTMLButtonElement;
	private statusEl: HTMLDivElement;
	private pollTimer: ReturnType<typeof setInterval> | null = null;
	private cleanupKasware: (() => void) | null = null;

	static get observedAttributes() {
		return ["amount", "asset", "seller", "memo", "api-key", "theme", "label"];
	}

	constructor() {
		super();
		this.state = {
			amount: "0",
			asset: "KAS",
			seller: "",
			memo: "",
			apiKey: "",
			theme: "dark",
			label: "Pay with KasWare",
			step: "ready",
			escrowId: null,
			error: null,
		};

		this.attachShadow({ mode: "open" });
		this.shadowRoot!.appendChild(template.content.cloneNode(true));
		this.btn = this.shadowRoot!.getElementById("btn") as HTMLButtonElement;
		this.statusEl = this.shadowRoot!.getElementById("status") as HTMLDivElement;

		this.btn.addEventListener("click", () => this.handlePay());
	}

	connectedCallback() {
		this.readAttributes();
		this.applyTheme();
		this.render();
	}

	disconnectedCallback() {
		this.stopPolling();
		if (this.cleanupKasware) {
			this.cleanupKasware();
			this.cleanupKasware = null;
		}
	}

	attributeChangedCallback() {
		this.readAttributes();
		this.applyTheme();
		this.render();
	}

	private readAttributes() {
		this.state.amount = this.getAttribute("amount") || "0";
		this.state.asset = (this.getAttribute("asset") || "KAS").toUpperCase();
		this.state.seller = this.getAttribute("seller") || "";
		this.state.memo = this.getAttribute("memo") || "";
		this.state.apiKey = this.getAttribute("api-key") || "";
		this.state.theme = this.getAttribute("theme") === "light" ? "light" : "dark";
		this.state.label = this.getAttribute("label") || "Pay with KasWare";
	}

	private applyTheme() {
		if (this.state.theme === "light") {
			this.style.setProperty("--dl-bg", "#ffffff");
			this.style.setProperty("--dl-text", "#1a1a2e");
			this.style.setProperty("--dl-border", "#ddd");
		} else {
			this.style.setProperty("--dl-bg", "#1a1a2e");
			this.style.setProperty("--dl-text", "#e0e0e0");
			this.style.setProperty("--dl-border", "#333");
		}
	}

	private render() {
		const s = this.state;
		this.btn.disabled = s.step !== "ready";
		this.btn.className = s.step === "ready" ? "primary" : s.step === "error" ? "outline" : "secondary";

		switch (s.step) {
			case "ready":
				this.btn.textContent = s.label;
				this.statusEl.textContent = "";
				this.statusEl.className = "status";
				break;
			case "connecting":
				this.btn.textContent = "Connecting…";
				this.statusEl.innerHTML = '<span class="spinner"></span>Connecting to KasWare…';
				this.statusEl.className = "status";
				break;
			case "funding":
				this.btn.textContent = "Creating escrow…";
				this.statusEl.innerHTML = '<span class="spinner"></span>Creating escrow…';
				this.statusEl.className = "status";
				break;
			case "waiting":
				this.btn.textContent = "Waiting for settlement…";
				this.statusEl.innerHTML =
					'<span class="spinner"></span>Escrow created! Waiting for settlement…';
				this.statusEl.className = "status";
				break;
			case "complete":
				this.btn.textContent = "✓ Complete";
				this.statusEl.textContent = "Payment complete!";
				this.statusEl.className = "status complete";
				break;
			case "error":
				this.btn.textContent = "Try Again";
				this.statusEl.textContent = s.error || "Something went wrong";
				this.statusEl.className = "status error";
				break;
		}
	}

	private setStep(step: PaymentState["step"]) {
		this.state.step = step;
		this.render();
	}

	private setError(msg: string) {
		this.state.step = "error";
		this.state.error = msg;
		this.render();
	}

	private emit(name: string, detail: Record<string, unknown>) {
		this.dispatchEvent(
			new CustomEvent(name, {
				bubbles: true,
				composed: true,
				detail,
			}),
		);
	}

	private async postJson<T>(path: string, body: unknown, apiKey?: string): Promise<T> {
		const headers: Record<string, string> = { "Content-Type": "application/json" };
		if (apiKey) headers["X-Daglock-Api-Key"] = apiKey;

		const controller = new AbortController();
		const timeout = setTimeout(() => controller.abort(), 30_000);
		try {
			const res = await fetch(API_BASE + path, {
				method: "POST",
				headers,
				body: JSON.stringify(body),
				signal: controller.signal,
			});
			if (!res.ok) {
				const text = await res.text();
				throw new Error(text || `HTTP ${res.status}`);
			}
			return res.json() as Promise<T>;
		} finally {
			clearTimeout(timeout);
		}
	}

	private async getJson<T>(path: string): Promise<T> {
		const controller = new AbortController();
		const timeout = setTimeout(() => controller.abort(), 30_000);
		try {
			const res = await fetch(API_BASE + path, { signal: controller.signal });
			if (!res.ok) {
				const text = await res.text();
				throw new Error(text || `HTTP ${res.status}`);
			}
			return res.json() as Promise<T>;
		} finally {
			clearTimeout(timeout);
		}
	}

	private stopPolling() {
		if (this.pollTimer) {
			clearInterval(this.pollTimer);
			this.pollTimer = null;
		}
	}

	private async pollStatus(escrowId: string) {
		this.stopPolling();
		return new Promise<void>((resolve) => {
			this.pollTimer = setInterval(async () => {
				try {
					const escrow: any = await this.getJson(`/v1/escrows/${escrowId}`);
					if (escrow.status === "settled" || escrow.status === "active") {
						this.stopPolling();
						this.state.step = "complete";
						this.state.escrowId = escrowId;
						this.render();
						this.emit("daglock-pay:complete", {
							escrowId,
							txId: escrow.lock_tx_id,
						});
						resolve();
					} else if (["refunded", "cancelled", "expired", "disputed"].includes(escrow.status)) {
						this.stopPolling();
						this.setError(`Escrow ${escrow.status}`);
						resolve();
					}
				} catch {
					// retry
				}
			}, 3000);
		});
	}

	async handlePay() {
		if (this.state.step === "complete") return;
		if (!this.state.seller) {
			this.setError("Missing seller address");
			return;
		}
		const sompi = Math.round(parseFloat(this.state.amount) * 1e8);
		if (sompi <= 0) {
			this.setError("Invalid amount");
			return;
		}

		this.setStep("connecting");

		try {
			const kasware = (window as any).kasware;
			if (!kasware) {
				this.setError("KasWare not found");
				this.statusEl.innerHTML =
					'<a href="https://kasware.com" target="_blank" rel="noopener noreferrer" class="link">Install KasWare extension</a>';
				return;
			}

			const accounts: string[] = await kasware.requestAccounts();
			if (!accounts || accounts.length === 0) {
				this.setError("No accounts available");
				return;
			}
			const buyerAddress = accounts[0];

			this.setStep("funding");

			const message = `daglock:create:${buyerAddress}:${Date.now()}`;
			const signature: string = await kasware.signMessage(message, "schnorr");
			const authHeaders = {
				"X-Daglock-Address": buyerAddress,
				"X-Daglock-Signature": signature,
				"X-Daglock-Message": message,
			};

			const escrowBody: Record<string, unknown> = {
				lock_tx_id: "pending",
				lock_tx_output_index: 0,
				buyer_address: buyerAddress,
				seller_address: this.state.seller,
				amount_sompi: sompi,
				asset_type: this.state.asset,
			};
			if (this.state.memo) escrowBody.memo = this.state.memo;

			const controller = new AbortController();
			const timeout = setTimeout(() => controller.abort(), 30_000);
			let escrowRes: Response;
			try {
				escrowRes = await fetch(API_BASE + "/v1/escrows", {
					method: "POST",
					headers: {
						"Content-Type": "application/json",
						...authHeaders,
					},
					body: JSON.stringify(escrowBody),
					signal: controller.signal,
				});
			} finally {
				clearTimeout(timeout);
			}

			if (!escrowRes.ok) {
				const text = await escrowRes.text();
				throw new Error(text || `HTTP ${escrowRes.status}`);
			}

			const escrowData: any = await escrowRes.json();
			const escrowId: string = escrowData.id;

			this.state.escrowId = escrowId;
			this.emit("daglock-pay:created", { escrowId });

			this.setStep("waiting");
			this.render();

			await this.pollStatus(escrowId);
		} catch (err) {
			this.setError((err as Error).message || "Payment failed");
		}
	}
}

customElements.define("daglock-pay", DaglockPay);

declare global {
	interface HTMLElementTagNameMap {
		"daglock-pay": DaglockPay;
	}
}
