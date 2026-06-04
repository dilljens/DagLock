import { useState } from "react";

export type LoadState<T> = {
	data?: T;
	error?: string;
	loading: boolean;
};

export function money(value: number | string | undefined): string {
	if (value === undefined) return "—";
	const numeric = typeof value === "string" ? Number.parseFloat(value) : value / 100_000_000;
	if (!Number.isFinite(numeric)) return "—";
	return `${numeric.toFixed(4)} KAS`;
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
	const seconds = Math.floor((Date.now() / 1000) - ts);
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

export function badge(status: string): string {
	return `pill pill-${status.replace(/_/g, "-")}`;
}
