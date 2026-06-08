import fetch from "cross-fetch";

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

function uuidFallback(): string {
  return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    return (c === "x" ? r : (r & 0x3) | 0x8).toString(16);
  });
}

export interface DaglockConfig {
  apiKey?: string;
  baseUrl?: string;
}

export interface Escrow {
  id: string;
  lockTxId: string;
  lockTxOutputIndex: number;
  status: string;
  assetType: string;
  buyerAddress: string;
  sellerAddress?: string | null;
  amountSompi: number;
  feeSompi: number;
  createdAt: number;
  settledAt?: number | null;
  refundedAt?: number | null;
  tradeHash?: string | null;
}

export interface CreateEscrowInput {
  lockTxId?: string;
  lockTxOutputIndex?: number;
  buyerAddress: string;
  sellerAddress?: string;
  amountSompi: number;
  expirationDaaScore?: number;
  assetType?: string;
  tradeHash?: string;
}

export interface Offer {
  id: string;
  creatorAddress: string;
  side: string;
  baseAsset: string;
  quoteAsset: string;
  amountSompi: number;
  status: string;
  createdAt: number;
}

export interface Reputation {
  address: string;
  tradeCount: number;
  score: number;
  settledCount: number;
  refundedCount: number;
  disputedCount: number;
  telegramHandle?: string | null;
  vouchesReceived: number;
}

export interface WebhookEvent {
  event: string;
  appId: string;
  createdAt: number;
  data: Record<string, unknown>;
}

/* --- Client --- */

function buildUrl(base: string, path: string): string {
  const b = base.replace(/\/+$/, "");
  const p = path.startsWith("/") ? path : `/${path}`;
  return `${b}${p}`;
}

export class Daglock {
  private baseUrl: string;
  private apiKey?: string;

  readonly escrows: EscrowApi;
  readonly offers: OfferApi;
  readonly reputation: ReputationApi;
  readonly webhooks: WebhookApi;

  constructor(config: DaglockConfig = {}) {
    this.baseUrl = config.baseUrl ?? "https://api.daglock.io/v1";
    this.apiKey = config.apiKey;
    this.escrows = new EscrowApi(this);
    this.offers = new OfferApi(this);
    this.reputation = new ReputationApi(this);
    this.webhooks = new WebhookApi();
  }

  async request<T>(method: string, path: string, body?: unknown): Promise<T> {
    const url = buildUrl(this.baseUrl, path);
    const headers: Record<string, string> = { "Content-Type": "application/json" };
    if (this.apiKey) headers["X-Daglock-Api-Key"] = this.apiKey;

    const res = await fetch(url, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    });
    if (!res.ok) {
      const text = await res.text().catch(() => "unknown error");
      throw new Error(`Daglock API error (${res.status}): ${text}`);
    }
    return res.json() as Promise<T>;
  }
}

/* --- Escrow API --- */

export class EscrowApi {
  constructor(private client: Daglock) {}

  async list(address: string, opts?: { role?: string; status?: string; limit?: number; offset?: number }): Promise<{ escrows: Escrow[]; total: number }> {
    let path = `/escrows?address=${encodeURIComponent(address)}`;
    if (opts?.role) path += `&role=${opts.role}`;
    if (opts?.status) path += `&status=${opts.status}`;
    if (opts?.limit) path += `&limit=${opts.limit}`;
    if (opts?.offset) path += `&offset=${opts.offset}`;
    return this.client.request("GET", path);
  }

  async get(id: string): Promise<Escrow> {
    return this.client.request("GET", `/escrows/${encodeURIComponent(id)}`);
  }

  async create(input: CreateEscrowInput): Promise<Escrow> {
    const body: Record<string, unknown> = {
      lock_tx_id: input.lockTxId ?? uuidFallback(),
      lock_tx_output_index: input.lockTxOutputIndex ?? 0,
      buyer_address: input.buyerAddress,
      amount_sompi: input.amountSompi,
    };
    if (input.sellerAddress) body.seller_address = input.sellerAddress;
    if (input.assetType) body.asset_type = input.assetType;
    if (input.expirationDaaScore) body.expiration_daa_score = input.expirationDaaScore;
    if (input.tradeHash) body.trade_hash = input.tradeHash;
    return this.client.request("POST", "/escrows", body);
  }
}

/* --- Offer API --- */

export class OfferApi {
  constructor(private client: Daglock) {}

  async list(opts?: { asset?: string; side?: string; status?: string }): Promise<{ offers: Offer[]; total: number }> {
    let path = "/offers?";
    if (opts?.asset) path += `asset=${opts.asset}&`;
    if (opts?.side) path += `side=${opts.side}&`;
    if (opts?.status) path += `status=${opts.status}&`;
    return this.client.request("GET", path);
  }

  async create(input: { creatorAddress: string; side: string; baseAsset: string; quoteAsset: string; amountSompi: number }): Promise<Offer> {
    return this.client.request("POST", "/offers", {
      creator_address: input.creatorAddress,
      side: input.side,
      base_asset: input.baseAsset,
      quote_asset: input.quoteAsset,
      amount_sompi: input.amountSompi,
    });
  }
}

/* --- Reputation API --- */

export class ReputationApi {
  constructor(private client: Daglock) {}

  async get(address: string): Promise<Reputation> {
    return this.client.request("GET", `/reputation/${encodeURIComponent(address)}`);
  }
}

/* --- Webhook verification --- */

export class WebhookApi {
  parseEvent(body: string): WebhookEvent {
    return JSON.parse(body) as WebhookEvent;
  }
}
