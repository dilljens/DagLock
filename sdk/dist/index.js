"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.WebhookApi = exports.ReputationApi = exports.OfferApi = exports.EscrowApi = exports.Daglock = void 0;
const cross_fetch_1 = __importDefault(require("cross-fetch"));
/**
 * DagLock SDK — TypeScript client for daglock.io
 *
 * ```typescript
 * import { Daglock } from "@daglock/sdk";
 * const dl = new Daglock({ apiKey: "dl_sk_..." });
 * const escrow = await dl.escrows.create({
 *   buyerAddress: "kaspa:...",
 *   amountSompi: 500_000_000_000,
 * });
 * ```
 */
function uuidFallback() {
    return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
        const r = (Math.random() * 16) | 0;
        return (c === "x" ? r : (r & 0x3) | 0x8).toString(16);
    });
}
/* --- Client --- */
function buildUrl(base, path) {
    const b = base.replace(/\/+$/, "");
    const p = path.startsWith("/") ? path : `/${path}`;
    return `${b}${p}`;
}
class Daglock {
    constructor(config = {}) {
        this.baseUrl = config.baseUrl ?? "https://api.daglock.io/v1";
        this.apiKey = config.apiKey;
        this.escrows = new EscrowApi(this);
        this.offers = new OfferApi(this);
        this.reputation = new ReputationApi(this);
        this.webhooks = new WebhookApi();
    }
    async request(method, path, body) {
        const url = buildUrl(this.baseUrl, path);
        const headers = { "Content-Type": "application/json" };
        if (this.apiKey)
            headers["X-Daglock-Api-Key"] = this.apiKey;
        const res = await (0, cross_fetch_1.default)(url, {
            method,
            headers,
            body: body ? JSON.stringify(body) : undefined,
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "unknown error");
            throw new Error(`Daglock API error (${res.status}): ${text}`);
        }
        return res.json();
    }
}
exports.Daglock = Daglock;
/* --- Escrow API --- */
class EscrowApi {
    constructor(client) {
        this.client = client;
    }
    async list(address, opts) {
        let path = `/escrows?address=${encodeURIComponent(address)}`;
        if (opts?.role)
            path += `&role=${opts.role}`;
        if (opts?.status)
            path += `&status=${opts.status}`;
        if (opts?.limit)
            path += `&limit=${opts.limit}`;
        if (opts?.offset)
            path += `&offset=${opts.offset}`;
        return this.client.request("GET", path);
    }
    async get(id) {
        return this.client.request("GET", `/escrows/${encodeURIComponent(id)}`);
    }
    async create(input) {
        const body = {
            lock_tx_id: input.lockTxId ?? uuidFallback(),
            lock_tx_output_index: input.lockTxOutputIndex ?? 0,
            buyer_address: input.buyerAddress,
            amount_sompi: input.amountSompi,
        };
        if (input.sellerAddress)
            body.seller_address = input.sellerAddress;
        if (input.assetType)
            body.asset_type = input.assetType;
        if (input.expirationDaaScore)
            body.expiration_daa_score = input.expirationDaaScore;
        if (input.tradeHash)
            body.trade_hash = input.tradeHash;
        return this.client.request("POST", "/escrows", body);
    }
}
exports.EscrowApi = EscrowApi;
/* --- Offer API --- */
class OfferApi {
    constructor(client) {
        this.client = client;
    }
    async list(opts) {
        let path = "/offers?";
        if (opts?.asset)
            path += `asset=${opts.asset}&`;
        if (opts?.side)
            path += `side=${opts.side}&`;
        if (opts?.status)
            path += `status=${opts.status}&`;
        return this.client.request("GET", path);
    }
    async create(input) {
        return this.client.request("POST", "/offers", {
            creator_address: input.creatorAddress,
            side: input.side,
            base_asset: input.baseAsset,
            quote_asset: input.quoteAsset,
            amount_sompi: input.amountSompi,
        });
    }
}
exports.OfferApi = OfferApi;
/* --- Reputation API --- */
class ReputationApi {
    constructor(client) {
        this.client = client;
    }
    async get(address) {
        return this.client.request("GET", `/reputation/${encodeURIComponent(address)}`);
    }
}
exports.ReputationApi = ReputationApi;
/* --- Webhook verification --- */
class WebhookApi {
    parseEvent(body) {
        return JSON.parse(body);
    }
}
exports.WebhookApi = WebhookApi;
//# sourceMappingURL=index.js.map