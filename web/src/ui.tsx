import { useState } from "react";
import type { LoadState } from "./helpers";

export function SectionTitle({ title, subtitle }: { title: string; subtitle: string }) {
	return (
		<div className="section-title">
			<h2>{title}</h2>
			<p>{subtitle}</p>
		</div>
	);
}

export function Panel({ title, children }: { title: string; children: React.ReactNode }) {
	return (
		<section className="panel">
			<div className="panel-head">
				<h3>{title}</h3>
			</div>
			{children}
		</section>
	);
}

export function LookupResult<T>({
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

export function FormField({ label, children }: { label: string; children: React.ReactNode }) {
	return (
		<label className="field">
			<span>{label}</span>
			{children}
		</label>
	);
}

export function ValidatedInput({
	label,
	value,
	onChange,
	placeholder,
	validate,
}: {
	label: string;
	value: string;
	onChange: (v: string) => void;
	placeholder: string;
	validate?: (v: string) => string | null;
}) {
	const [touched, setTouched] = useState(false);
	const trimmed = value.trim();
	const error = touched && validate ? validate(trimmed) : null;
	const valid = touched && !error && value.length > 0;
	return (
		<FormField label={label}>
			<div className="validated-input">
				<input
					value={value}
					onChange={(e) => onChange(e.target.value)}
					onBlur={() => setTouched(true)}
					placeholder={placeholder}
					className={error ? "input-error" : valid ? "input-valid" : ""}
				/>
				{error && <span className="input-feedback error">{error}</span>}
				{valid && <span className="input-feedback valid">OK</span>}
			</div>
		</FormField>
	);
}

export function kvad(addr: string): string | null {
	if (!addr.startsWith("kaspa:")) return "Must start with kaspa:";
	if (addr.length < 15) return "Address too short";
	return null;
}

export function ConfirmDialog({
	title,
	message,
	confirmLabel,
	onConfirm,
	onCancel,
}: {
	title: string;
	message: string;
	confirmLabel: string;
	onConfirm: () => void;
	onCancel: () => void;
}) {
	return (
		<div
			className="confirm-overlay"
			onClick={onCancel}
			onKeyDown={(e) => e.key === "Escape" && onCancel()}
		>
			<div
				className="confirm-dialog"
				onClick={(e) => e.stopPropagation()}
				onKeyDown={(e) => e.key === "Escape" && onCancel()}
			>
				<h3>{title}</h3>
				<p>{message}</p>
				<div className="confirm-actions">
					<button className="button" type="button" onClick={onCancel}>
						Cancel
					</button>
					<button className="button primary" type="button" onClick={onConfirm}>
						{confirmLabel}
					</button>
				</div>
			</div>
		</div>
	);
}

export function StatusTimeline({ status }: { status: string }) {
	const steps = [
		{ key: "pending_confirmation", label: "Locked" },
		{ key: "active", label: "Active" },
		{ key: "disputed", label: "Disputed", optional: true },
		{ key: "settled", label: "Settled" },
	];
	const currentIdx = steps.findIndex((s) => s.key === status);
	const altPath = ["refunded", "cancelled", "expired"];
	return (
		<div className="timeline">
			{steps.map((s, i) => (
				<div
					key={s.key}
					className={`timeline-step ${i <= currentIdx ? "active" : ""} ${s.optional ? "optional" : ""}`}
				>
					<div className="timeline-dot" />
					<span className="timeline-label">{s.label}</span>
				</div>
			))}
			{altPath.includes(status) && (
				<div className="timeline-step active alt">
					<div className="timeline-dot" />
					<span className="timeline-label">{status.charAt(0).toUpperCase() + status.slice(1)}</span>
				</div>
			)}
		</div>
	);
}
