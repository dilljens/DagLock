import { useState } from "react";

export type LoadState<T> = {
	data?: T;
	error?: string;
	loading: boolean;
};

/** Format sompi to KAS with locale-aware grouping and smart decimal places. */
export function money(value: number | string | undefined): string {
	if (value === undefined) return "—";
	const sompiVal = typeof value === "string" ? Number.parseFloat(value) : value;
	const kas = sompiVal / 100_000_000;
	if (!Number.isFinite(kas)) return "—";
	// Smart decimal places: 2 for round amounts, up to 8 for small amounts
	const decimals = kas >= 1 ? 2 : kas >= 0.001 ? 4 : 8;
	return `${kas.toLocaleString(undefined, { minimumFractionDigits: decimals, maximumFractionDigits: 8 })} KAS`;
}

/** Format sompi to compact form (1.2M KAS, 500K KAS, etc.) for dashboards. */
export function moneyCompact(value: number | undefined): string {
	if (!value) return "—";
	const kas = value / 100_000_000;
	if (kas >= 1_000_000) return `${(kas / 1_000_000).toFixed(1)}M KAS`;
	if (kas >= 1_000) return `${(kas / 1_000).toFixed(1)}K KAS`;
	return money(value);
}

/** Format a KAS decimal string (not sompi) with commas. */
export function formatKas(kas: number): string {
	return kas.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 8 });
}

export function sompi(kas: number): number {
	return Math.round(kas * 100_000_000);
}

export function time(value?: number | null): string {
	if (!value) return "—";
	return new Date(value * 1000).toLocaleString();
}

export function relativeTime(ts: number | null | undefined): string {
	if (!ts) return "—";
	const seconds = Math.floor(Date.now() / 1000 - ts);
	if (seconds < 60) return "just now";
	const minutes = Math.floor(seconds / 60);
	if (minutes < 60) return `${minutes}m ago`;
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return `${hours}h ago`;
	const days = Math.floor(hours / 24);
	if (days < 30) return `${days}d ago`;
	const months = Math.floor(days / 30);
	return `${months}mo ago`;
}

export function errMsg(err: unknown): string {
	if (err instanceof Error) return err.message;
	return String(err);
}

export function badge(status: string): string {
	return `pill pill-${status.replace(/_/g, "-")}`;
}
