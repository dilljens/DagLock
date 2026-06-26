// KasWare wallet integration for DagLock
// Detects KasWare browser extension and provides a unified API for:
//   - Connecting wallet
//   - Signing messages for authentication
//   - Sending KAS transactions

export type WalletState = {
	detected: boolean;
	connected: boolean;
	address: string | null;
	network: string | null;
	balance: string | null;
	loading: boolean;
	error: string | null;
	/** When true, a mock signature is used instead of real wallet signing (testnet dev mode). */
	manualMode: boolean;
};

/** Generate a dummy signature for testnet mock auth. */
export function mockSignature(message: string): string {
	// deterministic "signature" for dev mode — any hex string works with --mock-auth
	return "ff" + Array.from(new TextEncoder().encode(message))
		.map((b) => b.toString(16).padStart(2, "0"))
		.join("")
		.slice(0, 128)
		.padEnd(128, "0");
}

interface KaswareProvider {
	requestAccounts(): Promise<string[]>;
	getAccounts(): Promise<string[]>;
	getPublicKey(): Promise<string>;
	getBalance(): Promise<{ confirmed: number; pending: number }>;
	getNetwork(): Promise<string>;
	sendKaspa(to: string, sompi: number, opts?: { feeRate?: number }): Promise<string>;
	signMessage(message: string, type?: "ecdsa" | "schnorr"): Promise<string>;
	getVersion(): Promise<string>;
	on(event: string, handler: (data?: any) => void): void;
	removeListener(event: string, handler: (data?: any) => void): void;
}

declare global {
	interface Window {
		kasware?: KaswareProvider;
	}
}

// Detect KasWare with timeout
export async function detectKasware(timeoutMs = 3000): Promise<boolean> {
	return new Promise((resolve) => {
		if (window.kasware) {
			resolve(true);
			return;
		}
		const onInit = () => resolve(true);
		window.addEventListener("kasware#initialized", onInit, { once: true });
		setTimeout(() => {
			window.removeEventListener("kasware#initialized", onInit);
			resolve(!!window.kasware);
		}, timeoutMs);
	});
}

// Connect wallet - shows KasWare approval UI
export async function connectWallet(): Promise<{
	address: string;
	network: string;
	balance: string;
}> {
	if (!window.kasware) {
		throw new Error("KasWare wallet not detected. Install the KasWare browser extension.");
	}
	const accounts = await window.kasware.requestAccounts();
	if (!accounts || accounts.length === 0) {
		throw new Error("No accounts returned from wallet.");
	}
	const address = accounts[0];
	const network = await window.kasware.getNetwork();
	const balance = await window.kasware.getBalance();
	const balanceKas = (balance.confirmed / 1e8).toFixed(4);
	return { address, network, balance: balanceKas };
}

// Sign a message with KasWare - shows approval UI
export async function signMessage(
	message: string,
	type: "ecdsa" | "schnorr" = "schnorr",
): Promise<string> {
	if (!window.kasware) {
		throw new Error("KasWare wallet not detected.");
	}
	return window.kasware.signMessage(message, type);
}

// Subscribe to wallet events
export function subscribeToWallet(onStateChange: (state: WalletState) => void): () => void {
	const kasware = window.kasware;
	if (!kasware) return () => {};

	const onAccountsChanged = (accounts: string[]) => {
		if (accounts.length === 0) {
			onStateChange({
				detected: true,
				connected: false,
				address: null,
				network: null,
				balance: null,
				loading: false,
				error: null,
				manualMode: false,
			});
		}
	};
	const onNetworkChanged = (network: string) =>
		onStateChange({
			detected: true,
			connected: false,
			address: null,
			network,
			balance: null,
			loading: false,
			error: null,
			manualMode: false,
		});
	const onDisconnect = () => {
		onStateChange({
			detected: true,
			connected: false,
			address: null,
			network: null,
			balance: null,
			loading: false,
			error: null,
			manualMode: false,
		});
	};

	kasware.on("accountsChanged", onAccountsChanged);
	kasware.on("networkChanged", onNetworkChanged);
	kasware.on("disconnect", onDisconnect);
	return () => {
		kasware.removeListener("accountsChanged", onAccountsChanged);
		kasware.removeListener("networkChanged", onNetworkChanged);
		kasware.removeListener("disconnect", onDisconnect);
	};
}
