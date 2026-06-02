export type Health = {
  status: string;
  version: string;
  node_synced: boolean;
  node_daa_score: number;
  uptime_seconds: number;
};

export type NetworkInfo = {
  network: string;
  daa_score: number;
  block_count: number;
  difficulty: number;
  bps: number;
  daglock_kas_template_hash?: string | null;
  daglock_krc20_template_hash?: string | null;
};

export type Stats = {
  total_escrows: number;
  active_escrows: number;
  disputed_escrows: number;
  settled_escrows: number;
  refunded_escrows: number;
  cancelled_escrows: number;
  total_volume_kas: string;
  total_fees_collected_kas: string;
  unique_buyers: number;
  unique_sellers: number;
};

export type Offer = {
  id: string;
  creator_address: string;
  side: string;
  base_asset: string;
  quote_asset: string;
  amount_sompi: number;
  counterparty_address?: string | null;
  status: string;
  expires_at?: number | null;
  created_at: number;
};

export type Escrow = {
  id: string;
  lock_tx_id: string;
  lock_tx_output_index: number;
  status: string;
  asset_type: string;
  buyer_address: string;
  seller_address?: string | null;
  amount_sompi: number;
  fee_sompi: number;
  template_hash: number[];
  expiration_daa_score?: number | null;
  disputed_at?: number | null;
  dispute_reason?: string | null;
  cancelled_at?: number | null;
  expired_at?: number | null;
  created_at: number;
  settled_at?: number | null;
  refunded_at?: number | null;
};

export type Reputation = {
  address: string;
  trade_count: number;
  total_volume_sompi: number;
  settled_count: number;
  refunded_count: number;
  disputed_count: number;
  first_trade_at?: number | null;
  age_days: number;
  dispute_rate: number;
  refund_rate: number;
  score: number;
};

export type Receipt = {
  receipt_id: string;
  escrow_id: string;
  status: string;
  asset: string;
  amount_sompi: number;
  fee_sompi: number;
  buyer_address: string;
  seller_address?: string | null;
  lock_tx_id: string;
  lock_tx_output_index: number;
  expiration_daa_score?: number | null;
  disputed_at?: number | null;
  dispute_reason?: string | null;
  cancelled_at?: number | null;
  expired_at?: number | null;
  settled_at?: number | null;
  refunded_at?: number | null;
};

async function loadJson<T>(path: string): Promise<T> {
  const response = await fetch(path);
  if (!response.ok) {
    throw new Error(await response.text());
  }
  return response.json() as Promise<T>;
}

export const api = {
  health: () => loadJson<Health>('/v1/health'),
  network: () => loadJson<NetworkInfo>('/v1/network'),
  stats: () => loadJson<Stats>('/v1/stats'),
  offers: () => loadJson<{ offers: Offer[]; total: number }>('/v1/offers?status=proposed'),
  escrow: (id: string) => loadJson<Escrow>(`/v1/escrows/${encodeURIComponent(id)}`),
  reputation: (address: string) => loadJson<Reputation>(`/v1/reputation/${encodeURIComponent(address)}`),
  receipt: (id: string) => loadJson<Receipt>(`/v1/receipts/${encodeURIComponent(id)}`),
};
