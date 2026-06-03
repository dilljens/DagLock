import { useEffect, useMemo, useState } from 'react';
import { api, type AuthHeaders, type CreateEscrowRequest, type CreateOfferRequest, type Escrow, type Health, type NetworkInfo, type Offer, type Receipt, type Reputation, type Stats } from './api';

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

function sompi(kas: number): number {
  return Math.round(kas * 100_000_000);
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
  if (loading) return <p className="muted">Loading…</p>;
  if (error) return <p className="muted error-text">{error}</p>;
  if (!data) return <p className="muted">Enter an ID to inspect live state.</p>;
  return <div className="result">{render(data)}</div>;
}

function FormField({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="field">
      <span>{label}</span>
      {children}
    </label>
  );
}

/* ─── Create Offer ─── */
function CreateOfferForm({ onDone }: { onDone: () => void }) {
  const [side, setSide] = useState('sell');
  const [baseAsset, setBaseAsset] = useState('KAS');
  const [quoteAsset, setQuoteAsset] = useState('USDC');
  const [amount, setAmount] = useState('');
  const [address, setAddress] = useState('');
  const [counterparty, setCounterparty] = useState('');
  const [status, setStatus] = useState<'idle' | 'loading' | 'done' | 'error'>('idle');
  const [error, setError] = useState('');

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const amountNum = Number.parseFloat(amount);
    if (!amountNum || amountNum <= 0) return;
    if (!address.startsWith('kaspa:')) return;
    setStatus('loading');
    setError('');
    try {
      const body: CreateOfferRequest = {
        creator_address: address,
        side,
        base_asset: baseAsset,
        quote_asset: quoteAsset,
        amount_sompi: sompi(amountNum),
      };
      if (counterparty.startsWith('kaspa:')) body.counterparty_address = counterparty;
      await api.createOffer(body);
      setStatus('done');
      onDone();
    } catch (err) {
      setStatus('error');
      setError((err as Error).message);
    }
  }

  if (status === 'done') return <p className="muted success-text">Offer created!</p>;

  return (
    <form className="form form-stacked" onSubmit={handleSubmit}>
      <FormField label="Side">
        <select value={side} onChange={e => setSide(e.target.value)}>
          <option value="sell">Sell</option>
          <option value="buy">Buy</option>
        </select>
      </FormField>
      <FormField label="Sell asset">
        <input value={baseAsset} onChange={e => setBaseAsset(e.target.value)} placeholder="KAS" />
      </FormField>
      <FormField label="For asset">
        <input value={quoteAsset} onChange={e => setQuoteAsset(e.target.value)} placeholder="USDC" />
      </FormField>
      <FormField label="Amount (KAS)">
        <input type="number" step="any" value={amount} onChange={e => setAmount(e.target.value)} placeholder="100" />
      </FormField>
      <FormField label="Your address">
        <input value={address} onChange={e => setAddress(e.target.value)} placeholder="kaspa:..." />
      </FormField>
      <FormField label="Counterparty (optional)">
        <input value={counterparty} onChange={e => setCounterparty(e.target.value)} placeholder="kaspa:..." />
      </FormField>
      {error && <p className="muted error-text">{error}</p>}
      <button className="button primary" type="submit" disabled={status === 'loading'}>
        {status === 'loading' ? 'Creating…' : 'Create offer'}
      </button>
    </form>
  );
}

/* ─── Create Escrow ─── */
function CreateEscrowForm({ onDone }: { onDone: () => void }) {
  const [amount, setAmount] = useState('');
  const [buyerAddress, setBuyerAddress] = useState('');
  const [sellerAddress, setSellerAddress] = useState('');
  const [assetType, setAssetType] = useState('KAS');
  const [status, setStatus] = useState<'idle' | 'loading' | 'done' | 'error'>('idle');
  const [error, setError] = useState('');
  const [result, setResult] = useState<Escrow | null>(null);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const amountNum = Number.parseFloat(amount);
    if (!amountNum || amountNum <= 0) return;
    if (!buyerAddress.startsWith('kaspa:')) return;
    setStatus('loading');
    setError('');
    try {
      const body: CreateEscrowRequest = {
        lock_tx_id: crypto.randomUUID(),
        lock_tx_output_index: 0,
        buyer_address: buyerAddress,
        amount_sompi: sompi(amountNum),
        asset_type: assetType,
      };
      if (sellerAddress.startsWith('kaspa:')) body.seller_address = sellerAddress;
      const escrow = await api.createEscrow(body);
      setResult(escrow);
      setStatus('done');
      onDone();
    } catch (err) {
      setStatus('error');
      setError((err as Error).message);
    }
  }

  if (status === 'done' && result) {
    return (
      <div className="result stack">
        <p className="muted success-text">Escrow created!</p>
        <div className="row"><span>ID</span><code>{result.id}</code></div>
        <div className="row"><span>Amount</span><strong>{money(result.amount_sompi)}</strong></div>
      </div>
    );
  }

  return (
    <form className="form form-stacked" onSubmit={handleSubmit}>
      <FormField label="Asset type">
        <select value={assetType} onChange={e => setAssetType(e.target.value)}>
          <option value="KAS">KAS</option>
          <option value="KRC20">KRC-20</option>
        </select>
      </FormField>
      <FormField label="Amount (KAS)">
        <input type="number" step="any" value={amount} onChange={e => setAmount(e.target.value)} placeholder="100" />
      </FormField>
      <FormField label="Buyer address">
        <input value={buyerAddress} onChange={e => setBuyerAddress(e.target.value)} placeholder="kaspa:..." />
      </FormField>
      <FormField label="Seller address (optional)">
        <input value={sellerAddress} onChange={e => setSellerAddress(e.target.value)} placeholder="kaspa:..." />
      </FormField>
      {error && <p className="muted error-text">{error}</p>}
      <button className="button primary" type="submit" disabled={status === 'loading'}>
        {status === 'loading' ? 'Creating…' : 'Create escrow'}
      </button>
    </form>
  );
}

/* ─── Escrow Action (settle / refund / dispute / cancel) ─── */
type EscrowAction = 'settle' | 'refund' | 'dispute' | 'cancel';

function EscrowActionForm({ action }: { action: EscrowAction }) {
  const [escrowId, setEscrowId] = useState('');
  const [disputeReason, setDisputeReason] = useState('');
  const [authAddress, setAuthAddress] = useState('');
  const [authSignature, setAuthSignature] = useState('');
  const [status, setStatus] = useState<'idle' | 'loading' | 'done' | 'error'>('idle');
  const [error, setError] = useState('');
  const [result, setResult] = useState<{ status: string; escrow_id: string } | null>(null);

  const needsAuth = action === 'settle' || action === 'refund';
  const verb = action === 'settle' ? 'Settle' : action === 'refund' ? 'Refund' : action === 'dispute' ? 'Dispute' : 'Cancel';

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!escrowId) return;
    if (action === 'dispute' && !disputeReason) return;

    setStatus('loading');
    setError('');

    try {
      let auth: AuthHeaders | undefined;
      if (needsAuth) {
        if (!authAddress || !authSignature) {
          setStatus('error');
          setError('Address and signature are required for this action.');
          return;
        }
        auth = {
          address: authAddress,
          signature: authSignature,
          message: `${action}:${escrowId}`,
        };
      }

      let res: { status: string; escrow_id: string };
      switch (action) {
        case 'settle':
          res = await api.settleEscrow(escrowId, auth!);
          break;
        case 'refund':
          res = await api.refundEscrow(escrowId, auth!);
          break;
        case 'dispute':
          res = await api.disputeEscrow(escrowId, disputeReason);
          break;
        case 'cancel':
          res = await api.cancelEscrow(escrowId);
          break;
      }
      setResult(res);
      setStatus('done');
    } catch (err) {
      setStatus('error');
      setError((err as Error).message);
    }
  }

  if (status === 'done' && result) {
    return <p className="muted success-text">{result.status} — {result.escrow_id}</p>;
  }

  return (
    <form className="form form-stacked" onSubmit={handleSubmit}>
      <FormField label="Escrow ID">
        <input value={escrowId} onChange={e => setEscrowId(e.target.value)} placeholder="esc_..." />
      </FormField>
      {action === 'dispute' && (
        <FormField label="Reason">
          <input value={disputeReason} onChange={e => setDisputeReason(e.target.value)} placeholder="Why are you disputing?" />
        </FormField>
      )}
      {needsAuth && (
        <>
          <FormField label="Your address">
            <input value={authAddress} onChange={e => setAuthAddress(e.target.value)} placeholder="kaspa:..." />
          </FormField>
          <FormField label="Signature (hex)">
            <input value={authSignature} onChange={e => setAuthSignature(e.target.value)} placeholder="hex signature from wallet" />
          </FormField>
        </>
      )}
      {error && <p className="muted error-text">{error}</p>}
      <button className={`button ${action === 'cancel' ? '' : 'primary'}`} type="submit" disabled={status === 'loading'}>
        {status === 'loading' ? `${verb}ing…` : verb}
      </button>
    </form>
  );
}

/* ─── Offer item with accept/cancel inline ─── */
function OfferCard({ offer, onMutated }: { offer: Offer; onMutated: () => void }) {
  const [status, setStatus] = useState<'idle' | 'loading'>('idle');
  const [error, setError] = useState('');
  const [counterparty, setCounterparty] = useState('');

  async function handleAccept() {
    if (!counterparty.startsWith('kaspa:')) return;
    setStatus('loading');
    setError('');
    try {
      await api.acceptOffer(offer.id, counterparty);
      onMutated();
    } catch (err) {
      setError((err as Error).message);
      setStatus('idle');
    }
  }

  async function handleCancel() {
    setStatus('loading');
    setError('');
    try {
      await api.cancelOffer(offer.id);
      onMutated();
    } catch (err) {
      setError((err as Error).message);
      setStatus('idle');
    }
  }

  const canAct = offer.status === 'proposed';

  return (
    <article className="offer">
      <div className="offer-top">
        <strong>{offer.side.toUpperCase()} {money(offer.amount_sompi)}</strong>
        <span className={badge(offer.status)}>{offer.status}</span>
      </div>
      <p>{offer.base_asset} for {offer.quote_asset}</p>
      <code>{offer.id}</code>
      {canAct && (
        <div className="offer-actions">
          <input
            value={counterparty}
            onChange={e => setCounterparty(e.target.value)}
            placeholder="your kaspa address"
            className="offer-input"
          />
          <button className="button primary" disabled={status === 'loading'} onClick={handleAccept}>
            Accept
          </button>
          <button className="button" disabled={status === 'loading'} onClick={handleCancel}>
            Cancel
          </button>
        </div>
      )}
      {error && <p className="muted error-text">{error}</p>}
    </article>
  );
}

/* ─── Lookup panels ─── */
function EscrowLookup() {
  const [id, setId] = useState('');
  const [state, setState] = useState<LoadState<Escrow>>({ loading: false });

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!id) return;
    setState({ loading: true });
    try {
      setState({ data: await api.escrow(id.trim()), loading: false });
    } catch (err) {
      setState({ error: (err as Error).message, loading: false });
    }
  }

  return (
    <Panel title="Escrow lookup">
      <form className="form" onSubmit={handleSubmit}>
        <input value={id} onChange={e => setId(e.target.value)} placeholder="escrow id" />
        <button className="button primary" type="submit">Fetch</button>
      </form>
      <LookupResult loading={state.loading} error={state.error} data={state.data} render={data => (
        <div className="stack">
          <div className="row"><span>Status</span><strong><span className={badge(data.status)}>{data.status}</span></strong></div>
          <div className="row"><span>Amount</span><strong>{money(data.amount_sompi)}</strong></div>
          <div className="row"><span>Buyer</span><strong className="addr">{data.buyer_address}</strong></div>
          <div className="row"><span>Created</span><strong>{time(data.created_at)}</strong></div>
          {data.dispute_reason && <div className="row"><span>Dispute</span><strong>{data.dispute_reason}</strong></div>}
        </div>
      )} />
    </Panel>
  );
}

function ReputationLookup() {
  const [address, setAddress] = useState('');
  const [state, setState] = useState<LoadState<Reputation>>({ loading: false });

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!address) return;
    setState({ loading: true });
    try {
      setState({ data: await api.reputation(address.trim()), loading: false });
    } catch (err) {
      setState({ error: (err as Error).message, loading: false });
    }
  }

  return (
    <Panel title="Reputation">
      <form className="form" onSubmit={handleSubmit}>
        <input value={address} onChange={e => setAddress(e.target.value)} placeholder="kaspa address" />
        <button className="button" type="submit">Check</button>
      </form>
      <LookupResult loading={state.loading} error={state.error} data={state.data} render={data => (
        <div className="stack">
          <div className="row"><span>Trades</span><strong>{data.trade_count}</strong></div>
          <div className="row"><span>Dispute rate</span><strong>{(data.dispute_rate * 100).toFixed(1)}%</strong></div>
          <div className="row"><span>Score</span><strong>{data.score.toFixed(2)}/5</strong></div>
        </div>
      )} />
    </Panel>
  );
}

function ReceiptLookup() {
  const [id, setId] = useState('');
  const [state, setState] = useState<LoadState<Receipt>>({ loading: false });

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!id) return;
    setState({ loading: true });
    try {
      setState({ data: await api.receipt(id.trim()), loading: false });
    } catch (err) {
      setState({ error: (err as Error).message, loading: false });
    }
  }

  return (
    <Panel title="Receipt lookup">
      <form className="form" onSubmit={handleSubmit}>
        <input value={id} onChange={e => setId(e.target.value)} placeholder="escrow id" />
        <button className="button" type="submit">Fetch</button>
      </form>
      <LookupResult loading={state.loading} error={state.error} data={state.data} render={data => (
        <div className="stack">
          <div className="row"><span>ID</span><strong>{data.receipt_id}</strong></div>
          <div className="row"><span>Status</span><strong>{data.status}</strong></div>
          <div className="row"><span>Amount</span><strong>{money(data.amount_sompi)}</strong></div>
        </div>
      )} />
    </Panel>
  );
}

/* ─── Main App ─── */
export default function App() {
  const [health, setHealth] = useState<LoadState<Health>>({ loading: true });
  const [network, setNetwork] = useState<LoadState<NetworkInfo>>({ loading: true });
  const [stats, setStats] = useState<LoadState<Stats>>({ loading: true });
  const [offers, setOffers] = useState<LoadState<Offer[]>>({ loading: true });
  const [activeTab, setActiveTab] = useState<'create-offer' | 'create-escrow' | 'settle' | 'refund' | 'dispute' | 'cancel' | null>(null);

  function loadAll() {
    setHealth({ loading: true });
    setNetwork({ loading: true });
    setStats({ loading: true });
    setOffers({ loading: true });
    void Promise.all([
      api.health().then(data => setHealth({ data, loading: false })).catch(err => setHealth({ error: err.message, loading: false })),
      api.network().then(data => setNetwork({ data, loading: false })).catch(err => setNetwork({ error: err.message, loading: false })),
      api.stats().then(data => setStats({ data, loading: false })).catch(err => setStats({ error: err.message, loading: false })),
      api.offers().then(data => setOffers({ data: data.offers, loading: false })).catch(err => setOffers({ error: err.message, loading: false })),
    ]);
  }

  useEffect(loadAll, []);

  const highlights = useMemo(() => {
    const s = stats.data;
    return [
      ['Escrows', s?.total_escrows ?? '—'],
      ['Active', s?.active_escrows ?? '—'],
      ['Volume', s ? money(s.total_volume_kas) : '—'],
      ['Settled', s?.settled_escrows ?? '—'],
    ];
  }, [stats.data]);

  function closeTab() {
    setActiveTab(null);
  }

  const tabPanels: Record<string, { title: string; content: React.ReactNode }> = {
    'create-offer': { title: 'Create offer', content: <CreateOfferForm onDone={closeTab} /> },
    'create-escrow': { title: 'Create escrow', content: <CreateEscrowForm onDone={closeTab} /> },
    'settle': { title: 'Settle escrow', content: <EscrowActionForm action="settle" /> },
    'refund': { title: 'Refund escrow', content: <EscrowActionForm action="refund" /> },
    'dispute': { title: 'Dispute escrow', content: <EscrowActionForm action="dispute" /> },
    'cancel': { title: 'Cancel escrow', content: <EscrowActionForm action="cancel" /> },
  };

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
          <a href="#actions" className="button">Take action</a>
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
          {health.error || network.error ? (
            <p className="muted">{health.error || network.error}</p>
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
          {offers.error && <p className="muted error-text">{offers.error}</p>}
          {offers.data?.length === 0 && <p className="muted">No open offers right now. Create one below!</p>}
          {offers.data?.map(offer => (
            <OfferCard key={offer.id} offer={offer} onMutated={loadAll} />
          ))}
        </div>
      </section>

      <section id="actions" className="actions-section">
        <SectionTitle title="Actions" subtitle="Create offers & escrows, settle, refund, dispute, or cancel." />

        <div className="action-tabs">
          {([
            ['Create offer', 'create-offer'],
            ['Create escrow', 'create-escrow'],
            ['Settle', 'settle'],
            ['Refund', 'refund'],
            ['Dispute', 'dispute'],
            ['Cancel', 'cancel'],
          ] as const).map(([label, key]) => (
            <button
              key={key}
              className={`button ${activeTab === key ? 'primary' : ''}`}
              onClick={() => setActiveTab(activeTab === key ? null : key as typeof activeTab)}
            >
              {label}
            </button>
          ))}
        </div>

        {activeTab && (
          <div className="panel action-panel">
            <div className="panel-head">
              <h3>{tabPanels[activeTab].title}</h3>
              <button className="button" onClick={closeTab}>✕</button>
            </div>
            {tabPanels[activeTab].content}
          </div>
        )}
      </section>

      <section className="grid lookup-grid lookup-section">
        <EscrowLookup />
        <ReputationLookup />
        <ReceiptLookup />
      </section>
    </main>
  );
}
