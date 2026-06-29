import { useState, useEffect } from "react";
import { api } from "../api";
import { money, type LoadState } from "../helpers";
import { useWallet } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { Helmet } from "react-helmet-async";
import { EmptyState } from "../components/empty-state";

interface InvoiceData {
	id: string;
	freelancer_address: string;
	client_address: string | null;
	escrow_id: string | null;
	description: string;
	amount_sompi: number;
	due_date: number | null;
	status: string;
	created_at: number;
	paid_at: number | null;
	settled_at: number | null;
}

interface InvoiceResponse {
	invoice: InvoiceData;
	escrow_status: string | null;
	link: string;
}

export function PayInvoicePage() {
	// Parse invoice ID from URL path: /pay/INV_xxx
	const pathParts = window.location.pathname.split("/");
	const invoiceId = pathParts[2] || "";

	const [invoice, setInvoice] = useState<LoadState<InvoiceResponse>>({ loading: !!invoiceId });
	const { state: wallet, connect } = useWallet();
	const { notify } = useToast();
	const [paying, setPaying] = useState(false);

	useEffect(() => {
		if (!invoiceId) return;
		api
			.getInvoice(invoiceId)
			.then((data) => setInvoice({ data, loading: false }))
			.catch((err) => setInvoice({ error: err.message, loading: false }));
	}, [invoiceId]);

	async function handlePay() {
		if (!wallet.connected || !wallet.address) {
			await connect();
			return;
		}
		if (!invoice.data) return;
		setPaying(true);
		try {
			// Create an escrow linked to this invoice
			const escrow = await api.createEscrow({
				lock_tx_id: "pending",
				lock_tx_output_index: 0,
				buyer_address: wallet.address,
				amount_sompi: invoice.data.invoice.amount_sompi,
				invoice_id: invoice.data.invoice.id,
			});
			setInvoice((s) => {
				if (!s.data) return s;
				return {
					...s,
					data: {
						...s.data,
						invoice: { ...s.data.invoice, status: "paid", escrow_id: escrow.id },
					},
				};
			});
			notify("success", "Escrow created! Send KAS to lock it.");
		} catch (err) {
			notify("error", "Payment failed", (err as Error).message);
		} finally {
			setPaying(false);
		}
	}

	if (!invoiceId) {
		return (
			<EmptyState
				icon="📄"
				title="Invoice not found"
				description="No invoice ID in the URL."
			/>
		);
	}

	if (invoice.loading) return <p className="muted" style={{ padding: "2rem", textAlign: "center" }}>Loading invoice…</p>;
	if (invoice.error) return <p className="muted error-text" style={{ padding: "2rem", textAlign: "center" }}>{invoice.error}</p>;
	if (!invoice.data) return null;

	const inv = invoice.data.invoice;
	const statusColors: Record<string, string> = {
		draft: "#888",
		sent: "#ff9800",
		paid: "#4fc3f7",
		settled: "#53d769",
		disputed: "#ff7b7b",
		refunded: "#888",
		cancelled: "#888",
	};

	return (
		<>
			<Helmet>
				<title>Invoice — DagLock</title>
				<meta name="description" content={`Invoice for ${money(inv.amount_sompi)} KAS — ${inv.description}`} />
				<meta property="og:title" content={`Invoice for ${money(inv.amount_sompi)} KAS`} />
				<meta property="og:description" content={inv.description} />
				<meta property="og:type" content="website" />
			</Helmet>
			<div style={{ maxWidth: "560px", margin: "0 auto", padding: "2rem 1rem" }}>
				<div
					className="panel"
					style={{
						border: "1px solid var(--color-border)",
						borderRadius: "16px",
						padding: "2rem",
					}}
				>
					<div style={{ textAlign: "center", marginBottom: "1.5rem" }}>
						<h2 style={{ margin: 0, fontSize: "1.5rem" }}>INVOICE</h2>
						<code style={{ fontSize: "0.8rem", color: "#888" }}>{inv.id}</code>
					</div>

					<div
						style={{
							display: "flex",
							justifyContent: "space-between",
							alignItems: "center",
							padding: "1rem 0",
							borderTop: "1px solid var(--color-border)",
							borderBottom: "1px solid var(--color-border)",
						}}
					>
						<span style={{ fontSize: "0.9rem", color: "#888" }}>Amount</span>
						<strong style={{ fontSize: "1.3rem" }}>{money(inv.amount_sompi)} KAS</strong>
					</div>

					<div style={{ padding: "1rem 0", borderBottom: "1px solid var(--color-border)" }}>
						<span style={{ fontSize: "0.8rem", color: "#888" }}>Description</span>
						<p style={{ margin: "4px 0 0", fontSize: "0.95rem" }}>{inv.description}</p>
					</div>

					{inv.due_date && (
						<div style={{ padding: "1rem 0", borderBottom: "1px solid var(--color-border)" }}>
							<span style={{ fontSize: "0.8rem", color: "#888" }}>Due date</span>
							<p style={{ margin: "4px 0 0", fontSize: "0.9rem" }}>
								{new Date(inv.due_date * 1000).toLocaleDateString()}
							</p>
						</div>
					)}

					<div style={{ padding: "1rem 0", borderBottom: "1px solid var(--color-border)" }}>
						<span style={{ fontSize: "0.8rem", color: "#888" }}>From</span>
						<p style={{ margin: "4px 0 0", fontSize: "0.85rem", fontFamily: "monospace" }}>
							{inv.freelancer_address.slice(0, 16)}…
						</p>
					</div>

					<div style={{ padding: "1rem 0", borderBottom: "1px solid var(--color-border)" }}>
						<span style={{ fontSize: "0.8rem", color: "#888" }}>Status</span>
						<p style={{ margin: "4px 0 0" }}>
							<span
								style={{
									display: "inline-block",
									padding: "2px 10px",
									borderRadius: "12px",
									fontSize: "0.8rem",
									fontWeight: 600,
									background: `${statusColors[inv.status] || "#888"}22`,
									color: statusColors[inv.status] || "#888",
								}}
							>
								{inv.status.charAt(0).toUpperCase() + inv.status.slice(1)}
							</span>
						</p>
					</div>

					{(inv.status === "draft" || inv.status === "sent") && (
						<div style={{ marginTop: "1.5rem" }}>
							{wallet.connected ? (
								<button
									type="button"
									className="button primary"
									style={{ width: "100%", padding: "12px", fontSize: "1rem" }}
									disabled={paying}
									onClick={handlePay}
								>
									{paying ? "Creating escrow…" : `Pay ${money(inv.amount_sompi)} KAS`}
								</button>
							) : (
								<button
									type="button"
									className="button primary"
									style={{ width: "100%", padding: "12px", fontSize: "1rem" }}
									onClick={handlePay}
								>
									Connect Wallet to Pay
								</button>
							)}
							<p className="muted" style={{ fontSize: "0.75rem", textAlign: "center", marginTop: "8px" }}>
								Funds are held in a SilverScript covenant. You only pay when terms are met.
							</p>
						</div>
					)}

					{inv.status === "paid" && (
						<div style={{ marginTop: "1.5rem", textAlign: "center" }}>
							<p style={{ color: "#4fc3f7", fontWeight: 600 }}>✅ Paid — awaiting settlement</p>
						</div>
					)}

					{inv.status === "settled" && (
						<div style={{ marginTop: "1.5rem", textAlign: "center" }}>
							<p style={{ color: "#53d769", fontWeight: 600 }}>✅ Settled</p>
						</div>
					)}
				</div>
			</div>
		</>
	);
}
