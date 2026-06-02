import { useEffect, useMemo, useState } from 'react';
import { api, type Escrow, type Health, type NetworkInfo, type Offer, type Receipt, type Reputation, type Stats } from './api';

type LoadState<T> = {
  data?: T;
  error?: string;
  loading: boolean;
};

function money(value: number | string | undefined): string {
  if (value === undefined) return '—';
  const numeric = typeof value === 'string' ? Number.parseFloat(value) : value / 100_000_000;
  if (!Number.isFinite(numeric)) return '—';
  return `${numeric.toFixed(4)} KAS`;
}

function time(value?: number | null): string {
  if (!value) return '—';
  return new Date(value * 1000).toLocaleString();
}

function badge(status: string): string {
  return `pill pill-${status.replace(/_/g, '-')}`;
}

function SectionTitle({ title, subtitle }: { title: string; subtitle: string }) {
  return (
    <div className="section-title">
      <h2>{title}</h2>
      <p>{subtitle}</p>
    </div>
  );
}

function Panel({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="panel">
      <div className="panel-head">
        <h3>{title}</h3>
      </div>
      {children}
    </section>
  );
}

export default function App() {
  const [health, setHealth] = useState<LoadState<Health>>({ loading: true });
  const [network, setNetwork] = useState<LoadState<NetworkInfo>>({ loading: true });
  const [stats, setStats] = useState<LoadState<Stats>>({ loading: true });
  const [offers, setOffers] = useState<LoadState<Offer[]>>({ loading: true });
  const [escrowId, setEscrowId] = useState('');
  const [escrow, setEscrow] = useState<LoadState<Escrow>>({ loading: false });
  const [address, setAddress] = useState('');
  const [reputation, setReputation] = useState<LoadState<Reputation>>({ loading: false });
  const [receiptId, setReceiptId] = useState('');
  const [receipt, setReceipt] = useState<LoadState<Receipt>>({ loading: false });

  useEffect(() => {
    void Promise.all([
      api.health().then((data) => setHealth({ data, loading: false })).catch((error: Error) => setHealth({ error: error.message, loading: false })),
      api.network().then((data) => setNetwork({ data, loading: false })).catch((error: Error) => setNetwork({ error: error.message, loading: false })),
      api.stats().then((data) => setStats({ data, loading: false })).catch((error: Error) => setStats({ error: error.message, loading: false })),
      api.offers().then((data) => setOffers({ data: data.offers, loading: false })).catch((error: Error) => setOffers({ error: error.message, loading: false })),
    ]);
  }, []);

  const highlights = useMemo(() => {
    const rows = [
      ['Escrows', stats.data?.total_escrows ?? '—'],
      ['Active', stats.data?.active_escrows ?? '—'],
      ['Volume', stats.data ? money(stats.data.total_volume_kas) : '—'],
      ['Score', reputation.data ? reputation.data.score.toFixed(2) : '—'],
    ];
    return rows;
  }, [reputation.data, stats.data]);

  async function lookupEscrow(event: React.FormEvent) {
    event.preventDefault();
    setEscrow({ loading: true });
    try {
      setEscrow({ data: await api.escrow(escrowId.trim()), loading: false });
    } catch (error) {
      setEscrow({ error: (error as Error).message, loading: false });
    }
  }

  async function lookupReputation(event: React.FormEvent) {
    event.preventDefault();
    setReputation({ loading: true });
    try {
      setReputation({ data: await api.reputation(address.trim()), loading: false });
    } catch (error) {
      setReputation({ error: (error as Error).message, loading: false });
    }
  }

  async function lookupReceipt(event: React.FormEvent) {
    event.preventDefault();
    setReceipt({ loading: true });
    try {
      setReceipt({ data: await api.receipt(receiptId.trim()), loading: false });
    } catch (error) {
      setReceipt({ error: (error as Error).message, loading: false });
    }
  }

  return (
    <main className="app">
      <header className="hero">
        <div>
          <div className="brand">DagLock</div>
          <h1>Trustless escrow and atomic swaps on Kaspa.</h1>
          <p>
            The public front door for offers, escrows, reputation, and receipts on <strong>daglock.com</strong>.
          </p>
        </div>
        <div className="hero-actions">
          <a href="#offers" className="button primary">Browse offers</a>
          <a href="#lookup" className="button">Look up escrow</a>
        </div>
      </header>

      <section className="grid cards">
        {highlights.map(([label, value]) => (
          <article key={label} className="card">
            <span>{label}</span>
            <strong>{value}</strong>
          </article>
        ))}
      </section>

      <section className="grid two-up">
        <Panel title="Network">
          {health.error || network.error || stats.error ? (
            <p className="muted">{health.error || network.error || stats.error}</p>
          ) : (
            <div className="stack">
              <div className="row"><span>API</span><strong>{health.data?.status ?? '—'}</strong></div>
              <div className="row"><span>Network</span><strong>{network.data?.network ?? '—'}</strong></div>
              <div className="row"><span>Version</span><strong>{health.data?.version ?? '—'}</strong></div>
              <div className="row"><span>Fee tier</span><strong>0.5%</strong></div>
            </div>
          )}
        </Panel>

        <Panel title="Public stats">
          {stats.data ? (
            <div className="stack">
              <div className="row"><span>Total escrows</span><strong>{stats.data.total_escrows}</strong></div>
              <div className="row"><span>Settled</span><strong>{stats.data.settled_escrows}</strong></div>
              <div className="row"><span>Disputed</span><strong>{stats.data.disputed_escrows}</strong></div>
              <div className="row"><span>Fees</span><strong>{money(stats.data.total_fees_collected_kas)}</strong></div>
            </div>
          ) : (
            <p className="muted">Loading stats…</p>
          )}
        </Panel>
      </section>

      <section id="offers">
        <SectionTitle title="Open offers" subtitle="Public listings available to counterparties." />
        <div className="offers">
          {offers.loading && <p className="muted">Loading offers…</p>}
          {offers.error && <p className="muted">{offers.error}</p>}
          {offers.data?.length === 0 && <p className="muted">No open offers right now.</p>}
          {offers.data?.map((offer) => (
            <article key={offer.id} className="offer">
              <div className="offer-top">
                <strong>{offer.side.toUpperCase()} {money(offer.amount_sompi)}</strong>
                <span className={badge(offer.status)}>{offer.status}</span>
              </div>
              <p>{offer.base_asset} for {offer.quote_asset}</p>
              <code>{offer.id}</code>
            </article>
          ))}
        </div>
      </section>

      <section id="lookup" className="grid lookup-grid">
        <Panel title="Escrow lookup">
          <form className="form" onSubmit={lookupEscrow}>
            <input value={escrowId} onChange={(event) => setEscrowId(event.target.value)} placeholder="escrow id" />
            <button className="button primary" type="submit">Fetch escrow</button>
          </form>
          <LookupResult loading={escrow.loading} error={escrow.error} data={escrow.data} render={(data) => (
            <div className="stack">
              <div className="row"><span>Status</span><strong><span className={badge(data.status)}>{data.status}</span></strong></div>
              <div className="row"><span>Amount</span><strong>{money(data.amount_sompi)}</strong></div>
              <div className="row"><span>Buyer</span><strong>{data.buyer_address}</strong></div>
              <div className="row"><span>Created</span><strong>{time(data.created_at)}</strong></div>
              <div className="row"><span>Dispute</span><strong>{data.dispute_reason ?? '—'}</strong></div>
            </div>
          )} />
        </Panel>

        <Panel title="Reputation & receipts">
          <form className="form" onSubmit={lookupReputation}>
            <input value={address} onChange={(event) => setAddress(event.target.value)} placeholder="kaspa address" />
            <button className="button primary" type="submit">Fetch reputation</button>
          </form>
          <LookupResult loading={reputation.loading} error={reputation.error} data={reputation.data} render={(data) => (
            <div className="stack">
              <div className="row"><span>Trades</span><strong>{data.trade_count}</strong></div>
              <div className="row"><span>Dispute rate</span><strong>{(data.dispute_rate * 100).toFixed(1)}%</strong></div>
              <div className="row"><span>Refund rate</span><strong>{(data.refund_rate * 100).toFixed(1)}%</strong></div>
              <div className="row"><span>Score</span><strong>{data.score.toFixed(2)}/5</strong></div>
            </div>
          )} />

          <form className="form" onSubmit={lookupReceipt}>
            <input value={receiptId} onChange={(event) => setReceiptId(event.target.value)} placeholder="escrow id for receipt" />
            <button className="button" type="submit">Fetch receipt</button>
          </form>
          <LookupResult loading={receipt.loading} error={receipt.error} data={receipt.data} render={(data) => (
            <div className="stack">
              <div className="row"><span>Receipt</span><strong>{data.receipt_id}</strong></div>
              <div className="row"><span>Status</span><strong>{data.status}</strong></div>
              <div className="row"><span>Verified</span><strong>{data.expired_at ? 'Expired' : 'Active'}</strong></div>
            </div>
          )} />
        </Panel>
      </section>
    </main>
  );
}

function LookupResult<T>({
  loading,
  error,
  data,
  render,
}: {
  loading: boolean;
  error?: string;
  data?: T;
  render: (data: T) => React.ReactNode;
}) {
  if (loading) {
    return <p className="muted">Loading…</p>;
  }
  if (error) {
    return <p className="muted">{error}</p>;
  }
  if (!data) {
    return <p className="muted">Enter an ID to inspect live state.</p>;
  }
  return <div className="result">{render(data)}</div>;
}
