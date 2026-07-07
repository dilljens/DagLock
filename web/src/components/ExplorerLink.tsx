import { useState, useEffect } from "react";
import { api } from "../api";

let cachedBaseUrl: string | null = null;
let fetchPromise: Promise<string> | null = null;

async function getExplorerBaseUrl(): Promise<string> {
	if (cachedBaseUrl) return cachedBaseUrl;
	if (!fetchPromise) {
		fetchPromise = api
			.explorer()
			.then((d) => {
				cachedBaseUrl = d.base_url;
				return d.base_url;
			})
			.catch(() => {
				cachedBaseUrl = "https://kas.fyi";
				return cachedBaseUrl;
			});
	}
	return fetchPromise;
}

/**
 * Links to a Kaspa block explorer for a given TX hash.
 */
export function ExplorerTxLink({ txid, label }: { txid: string; label?: string }) {
	const [baseUrl, setBaseUrl] = useState("https://kas.fyi");

	useEffect(() => {
		getExplorerBaseUrl().then(setBaseUrl);
	}, []);

	const display = label || `${txid.slice(0, 16)}…`;
	return (
		<a
			href={`${baseUrl}/transaction/${txid}`}
			target="_blank"
			rel="noopener noreferrer"
			className="explorer-link"
		>
			🔗 {display}
		</a>
	);
}

/**
 * Links to a Kaspa block explorer for a given address.
 */
export function ExplorerAddressLink({ address, label }: { address: string; label?: string }) {
	const [baseUrl, setBaseUrl] = useState("https://kas.fyi");

	useEffect(() => {
		getExplorerBaseUrl().then(setBaseUrl);
	}, []);

	const display = label || `${address.slice(0, 16)}…`;
	return (
		<a
			href={`${baseUrl}/address/${address}`}
			target="_blank"
			rel="noopener noreferrer"
			className="explorer-link"
		>
			🔗 {display}
		</a>
	);
}

/**
 * Links to a Kaspa block explorer for an escrow (uses lock_tx_id).
 */
export function ExplorerEscrowLink({ escrowId }: { escrowId: string }) {
	const [baseUrl, setBaseUrl] = useState("https://kas.fyi");

	useEffect(() => {
		getExplorerBaseUrl().then(setBaseUrl);
	}, []);

	return (
		<a
			href={`${baseUrl}/transaction/${escrowId}`}
			target="_blank"
			rel="noopener noreferrer"
			className="explorer-link"
		>
			🔗 View on Explorer
		</a>
	);
}
