/**
 * Kaspa wallet deep link utilities.
 *
 * Supports:
 * - `kaspa:` URI scheme (BIP-21 style) — works with KasWare, Kaspium
 * - `kasware:` custom protocol — KasWare specific (if available)
 *
 * Reference:
 * - Kaspa URI spec: https://github.com/kaspa-labs/kaspa-docs
 * - KasWare: Chrome extension, injects window.kasware
 * - Kaspium: Mobile wallet, handles kaspa: URIs natively
 */

/**
 * Generate a kaspa: URI for sending KAS.
 * Example: kaspa:qabc...xyz?amount=1000
 */
export function kaspaSendUri(
	address: string,
	amountSompi: number,
	memo?: string,
): string {
	const kasAmount = (amountSompi / 100_000_000).toFixed(8);
	let uri = `kaspa:${address}?amount=${kasAmount}`;
	if (memo) uri += `&memo=${encodeURIComponent(memo)}`;
	return uri;
}

/**
 * Detect if the current browser has KasWare installed.
 */
export function hasKasWare(): boolean {
	return typeof window !== "undefined" && "kasware" in window;
}

/**
 * Open a deep link. Tries KasWare first, falls back to kaspa: URI.
 */
export function openDeepLink(uri: string): void {
	if (hasKasWare()) {
		// KasWare listens for this custom event
		try {
			(window as any).kasware?.sendKaspa?.(uri);
			return;
		} catch {
			// fall through
		}
	}
	// Fallback: open kaspa: URI (may not work in all browsers)
	window.open(uri, "_blank");
}

/**
 * Generate a deep link to open a specific escrow in KasWare/Kaspium.
 */
export function escrowDeepLink(
	escrowId: string,
	action: "settle" | "refund" | "dispute",
): string {
	const baseUrl = import.meta.env.VITE_APP_URL || "https://daglock.com";
	return `${baseUrl}/escrows?id=${escrowId}&action=${action}`;
}
