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
	skeleton,
}: {
	loading: boolean;
	error?: string;
	data?: T;
	render: (data: T) => React.ReactNode;
	skeleton?: React.ReactNode;
}) {
	if (loading) return skeleton || <div className="skeleton-card"><div className="skeleton" style={{ height: 48 }} /></div>;
	if (error) return <p className="muted error-text">{error}</p>;
	if (!data) return <p className="muted" style={{ textAlign: "center", padding: "32px 0" }}>Enter an ID to inspect live state.</p>;
	return <div className="result">{render(data)}</div>;
}

export function SkeletonStats() {
	return (
		<div className="skeleton-stats">
			{Array.from({ length: 4 }).map((_, i) => <div key={i} className="skeleton" />)}
		</div>
	);
}

export function SkeletonOffers() {
	return (
		<div className="skeleton-offers">
			{Array.from({ length: 3 }).map((_, i) => (
				<div key={i} className="skeleton-card">
					<div className="skeleton-row">
						<div className="skeleton skeleton-text">
							<div className="skeleton" />
							<div className="skeleton" />
						</div>
					</div>
					<div className="skeleton-text">
						<div className="skeleton" style={{ width: "80%" }} />
						<div className="skeleton" style={{ width: "50%" }} />
					</div>
				</div>
			))}
		</div>
	);
}

export function SkeletonTable({ rows = 5 }: { rows?: number }) {
	return (
		<div className="skeleton-table">
			{Array.from({ length: rows }).map((_, i) => <div key={i} className="skeleton" />)}
		</div>
	);
}

export function SkeletonHero() {
	return (
		<div className="hero" style={{ height: 180 }}>
			<div className="skeleton-text" style={{ gap: 12 }}>
				<div className="skeleton" style={{ height: 32, width: "40%" }} />
				<div className="skeleton" style={{ height: 16, width: "70%" }} />
				<div className="skeleton" style={{ height: 16, width: "50%" }} />
			</div>
			<div className="skeleton" style={{ width: 140, height: 48, borderRadius: 999 }} />
		</div>
	);
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
		<ConfirmDialogInner title={title} message={message} confirmLabel={confirmLabel} onConfirm={onConfirm} onCancel={onCancel} />
	);
}

/* ─── Radix Dialog wrapper ─── */
import * as DialogPrimitive from "@radix-ui/react-dialog";

export function Dialog({
	open,
	onOpenChange,
	title,
	description,
	children,
}: {
	open: boolean;
	onOpenChange: (open: boolean) => void;
	title: string;
	description?: string;
	children: React.ReactNode;
}) {
	return (
		<DialogPrimitive.Root open={open} onOpenChange={onOpenChange}>
			<DialogPrimitive.Portal>
				<DialogPrimitive.Overlay className="dialog-overlay" />
				<DialogPrimitive.Content className="dialog-content" aria-describedby={description ? "dialog-desc" : undefined}>
					<DialogPrimitive.Title className="dialog-title">{title}</DialogPrimitive.Title>
					{description && <DialogPrimitive.Description id="dialog-desc" className="dialog-description">{description}</DialogPrimitive.Description>}
					{children}
					<DialogPrimitive.Close asChild>
						<button className="dialog-close" aria-label="Close">✕</button>
					</DialogPrimitive.Close>
				</DialogPrimitive.Content>
			</DialogPrimitive.Portal>
		</DialogPrimitive.Root>
	);
}

/* Backward-compatible ConfirmDialog using Radix */
function ConfirmDialogInner({
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
		<Dialog open={true} onOpenChange={(open) => { if (!open) onCancel(); }} title={title} description={message}>
			<div className="confirm-actions" style={{ marginTop: 20 }}>
				<DialogPrimitive.Close asChild>
					<button className="button" type="button">Cancel</button>
				</DialogPrimitive.Close>
				<button className="button primary" type="button" onClick={onConfirm}>
					{confirmLabel}
				</button>
			</div>
		</Dialog>
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
