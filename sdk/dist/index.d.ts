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
export declare class Daglock {
    private baseUrl;
    private apiKey?;
    readonly escrows: EscrowApi;
    readonly offers: OfferApi;
    readonly reputation: ReputationApi;
    readonly webhooks: WebhookApi;
    constructor(config?: DaglockConfig);
    request<T>(method: string, path: string, body?: unknown): Promise<T>;
}
export declare class EscrowApi {
    private client;
    constructor(client: Daglock);
    list(address: string, opts?: {
        role?: string;
        status?: string;
        limit?: number;
        offset?: number;
    }): Promise<{
        escrows: Escrow[];
        total: number;
    }>;
    get(id: string): Promise<Escrow>;
    create(input: CreateEscrowInput): Promise<Escrow>;
}
export declare class OfferApi {
    private client;
    constructor(client: Daglock);
    list(opts?: {
        asset?: string;
        side?: string;
        status?: string;
    }): Promise<{
        offers: Offer[];
        total: number;
    }>;
    create(input: {
        creatorAddress: string;
        side: string;
        baseAsset: string;
        quoteAsset: string;
        amountSompi: number;
    }): Promise<Offer>;
}
export declare class ReputationApi {
    private client;
    constructor(client: Daglock);
    get(address: string): Promise<Reputation>;
}
export declare class WebhookApi {
    parseEvent(body: string): WebhookEvent;
}
//# sourceMappingURL=index.d.ts.map