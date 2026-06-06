import { useState } from "react";
import { api, type CompileResponse } from "../api";
import { FormField } from "../ui";

/* ─── Compile Covenant Form ─── */
export function CompileCovenantForm({ onDone }: { onDone: () => void }) {
	const [template, setTemplate] = useState("daglock");
	const [paramsStr, setParamsStr] = useState("{}");
	const [status, setStatus] = useState<"idle" | "loading" | "done" | "error">("idle");
	const [error, setError] = useState("");
	const [result, setResult] = useState<CompileResponse | null>(null);

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		let params: Record<string, string>;
		try {
			params = JSON.parse(paramsStr);
		} catch {
			setError("Params must be valid JSON");
			return;
		}
		setStatus("loading");
		setError("");
		try {
			const r = await api.compile(template, params);
			setResult(r);
			setStatus("done");
			onDone();
		} catch (err) {
			setStatus("error");
			setError((err as Error).message);
		}
	}

	return (
		<form className="form form-stacked" onSubmit={handleSubmit}>
			<FormField label="Template">
				<select value={template} onChange={(e) => setTemplate(e.target.value)}>
					<option value="daglock">DagLock (KAS escrow)</option>
					<option value="daglock_arbiter">DagLock Arbiter (with mediator)</option>
					<option value="daglock_vault">DagLock Vault (time-locked)</option>
				</select>
			</FormField>
			<FormField label="Params (JSON)">
				<textarea
					value={paramsStr}
					onChange={(e) => setParamsStr(e.target.value)}
					className="evidence-input"
					placeholder='{"buyer_key":"...","seller_key":"...","timeout":"1700000000","treasury_key":"..."}'
				/>
			</FormField>
			{error && <p className="muted error-text">{error}</p>}
			<button className="button primary" type="submit" disabled={status === "loading"}>
				{status === "loading" ? "Compiling…" : "Compile"}
			</button>
			{result && (
				<pre className="muted" style={{ fontSize: "0.7rem", marginTop: 8 }}>
					{JSON.stringify(result, null, 2)}
				</pre>
			)}
		</form>
	);
}
