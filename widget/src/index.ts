/**
 * @daglock/widget — Embeddable escrow web components.
 *
 * Usage:
 *   <script src="https://unpkg.com/@daglock/widget"></script>
 *   <daglock-escrow
 *     amount="5000"
 *     asset="KAS"
 *     buyer-address="kaspa:..."
 *     seller-address="kaspa:..."
 *   ></daglock-escrow>
 */

const API_BASE = "https://api.daglock.io/v1";

function css() {
  return `
    :host { display: block; font-family: system-ui, sans-serif; }
    .card { border: 1px solid #2a4a2a; border-radius: 16px; padding: 20px;
            background: #0a1a0a; color: #e0f0e0; }
    .card h3 { margin: 0 0 12px; font-size: 16px; }
    .row { display: flex; justify-content: space-between; padding: 8px 0;
           border-bottom: 1px solid rgba(83, 215, 105, 0.12); font-size: 14px; }
    .row:last-child { border-bottom: 0; }
    .label { color: #88b888; }
    .value { font-weight: 600; }
    .btn { display: inline-flex; align-items: center; justify-content: center;
           gap: 8px; border-radius: 999px; border: 1px solid rgba(83, 215, 105, 0.2);
           background: rgba(83, 215, 105, 0.06); color: #e0f0e0;
           padding: 10px 18px; font-weight: 600; cursor: pointer;
           font-size: 14px; width: 100%; margin-top: 12px; }
    .btn-primary { background: #53d769; color: #000; border-color: transparent; }
    .btn:disabled { opacity: 0.5; cursor: not-allowed; }
    .status { text-align: center; font-size: 13px; margin-top: 8px; color: #88b888; }
    .error { color: #ff7b7b; text-align: center; font-size: 13px; margin-top: 8px; }
  `;
}

class DaglockEscrowElement extends HTMLElement {
  private shadow: ShadowRoot;

  constructor() {
    super();
    this.shadow = this.attachShadow({ mode: "closed" });
  }

  connectedCallback() {
    this.render();
  }

  get amount(): string { return this.getAttribute("amount") || "0"; }
  get asset(): string { return this.getAttribute("asset") || "KAS"; }
  get buyerAddress(): string { return this.getAttribute("buyer-address") || ""; }
  get sellerAddress(): string { return this.getAttribute("seller-address") || ""; }

  async handleCreate() {
    const btn = this.shadow.querySelector(".btn") as HTMLButtonElement;
    if (btn) btn.disabled = true;

    const statusEl = this.shadow.querySelector(".status") as HTMLElement;
    const errorEl = this.shadow.querySelector(".error") as HTMLElement;
    if (statusEl) statusEl.textContent = "Creating escrow...";
    if (errorEl) errorEl.textContent = "";

    try {
      const sompi = Math.round(parseFloat(this.amount) * 100_000_000);
      const body = {
        lock_tx_id: crypto.randomUUID ? crypto.randomUUID() : self.crypto.randomUUID(),
        lock_tx_output_index: 0,
        buyer_address: this.buyerAddress,
        amount_sompi: sompi,
        asset_type: this.asset,
      };
      if (this.sellerAddress) Object.assign(body, { seller_address: this.sellerAddress });

      const res = await fetch(`${API_BASE}/escrows`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });
      if (!res.ok) throw new Error(await res.text());

      const escrow = await res.json();
      if (statusEl) statusEl.textContent = "Escrow created! ID: " + escrow.id;

      this.dispatchEvent(new CustomEvent("created", { detail: escrow }));
    } catch (err) {
      if (errorEl) errorEl.textContent = (err as Error).message;
      if (statusEl) statusEl.textContent = "";
    } finally {
      if (btn) btn.disabled = false;
    }
  }

  render() {
    this.shadow.innerHTML = `
      <style>${css()}</style>
      <div class="card">
        <h3>🔒 DagLock Escrow</h3>
        <div class="row">
          <span class="label">Amount</span>
          <span class="value">${this.amount} ${this.asset}</span>
        </div>
        <div class="row">
          <span class="label">Buyer</span>
          <span class="value">${this.buyerAddress.slice(0, 16)}...</span>
        </div>
        ${this.sellerAddress ? `<div class="row"><span class="label">Seller</span><span class="value">${this.sellerAddress.slice(0, 16)}...</span></div>` : ""}
        <button class="btn btn-primary" part="create-btn">Create Escrow</button>
        <div class="status"></div>
        <div class="error"></div>
      </div>
    `;
    this.shadow.querySelector(".btn")?.addEventListener("click", () => this.handleCreate());
  }
}

customElements.define("daglock-escrow", DaglockEscrowElement);
