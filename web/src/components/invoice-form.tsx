import { useState } from "react";
import { api } from "../api";
import { useWallet } from "../context/WalletContext";
import { useToast } from "../layout/Toast";
import { FormField } from "../ui";

export function CreateInvoiceForm() {
	const { state: wallet, sign } = useWallet();
	const { notify } = useToast();
	const [description, setDescription] = useState("");
	const [amount, setAmount] = useState("");
	const [dueDate, setDueDate] = useState("");
	const [status, setStatus] = useState<"idle" | "loading" | "done">("idle");
	const [invoiceLink, setInvoiceLink] = useState("");

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		const amountNum = Number.parseFloat(amount);
		if (!amountNum || amountNum <= 0 || !description.trim()) return;
		if (!wallet.address) return;

		setStatus("loading");
		try {
			const sompiAmount = Number((amountNum * 100_000_000).toFixed(0));
			const msg = `create:invoice:${wallet.address}`;
			const sig = await sign(msg);
			const auth = { address: wallet.address, signature: sig, message: msg };

			const dueTs = dueDate ? Math.floor(new Date(dueDate).getTime() / 1000) : undefined;

			const result = await api.createInvoice(
				{ description: description.trim(), amount_sompi: sompiAmount, due_date: dueTs },
				auth,
			);
			setInvoiceLink(result.link);
			setStatus("done");
			notify("success", "Invoice created!");
		} catch (err) {
			notify("error", "Failed to create invoice", (err as Error).message);
			setStatus("idle");
		}
	}

	if (status === "done") {
		return (
			<div className="panel" style={{ padding: "1.5rem", textAlign: "center" }}>
				<p style={{ color: "#53d769", fontWeight: 600, marginBottom: "12px" }}>
					✅ Invoice created!
				</p>
				<div
					style={{
						background: "rgba(0,0,0,0.3)",
						padding: "12px",
						borderRadius: "8px",
						fontSize: "13px",
						wordBreak: "break-all",
					}}
				>
					<code>{invoiceLink}</code>
				</div>
				<div style={{ marginTop: "12px", display: "flex", gap: "8px", justifyContent: "center" }}>
					<button
						type="button"
						className="button"
						onClick={() => {
							navigator.clipboard.writeText(invoiceLink);
							notify("success", "Link copied!");
						}}
					>
						Copy link
					</button>
					<button
						type="button"
						className="button primary"
						onClick={() => {
							setStatus("idle");
							setDescription("");
							setAmount("");
							setDueDate("");
							setInvoiceLink("");
						}}
					>
						Create another
					</button>
				</div>
			</div>
		);
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<p className="muted" style={{ fontSize: "13px", marginBottom: "12px" }}>
				Create a shareable invoice. When the client pays, an escrow is created automatically.
			</p>
			<FormField label="Description">
				<input
					value={description}
					onChange={(e) => setDescription(e.target.value)}
					placeholder="Website redesign — Phase 1"
				/>
			</FormField>
			<FormField label="Amount (KAS)">
				<input
					type="number"
					step="any"
					value={amount}
					onChange={(e) => setAmount(e.target.value)}
					placeholder="500"
				/>
			</FormField>
			<FormField label="Due date (optional)">
				<input type="date" value={dueDate} onChange={(e) => setDueDate(e.target.value)} />
			</FormField>
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Creating…" : "Create Invoice"}
			</button>
		</form>
	);
}
