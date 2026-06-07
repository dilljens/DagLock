// Wallet context for DagLock — provides connected wallet state to all components.
// Eliminates the need for manual "your address" inputs everywhere.

import { createContext, useContext, useState, useEffect, useCallback, type ReactNode } from "react";
import { detectKasware, connectWallet, signMessage, subscribeToWallet, type WalletState } from "../kasware";

export interface WalletContextValue {
	state: WalletState;
	connect: () => Promise<void>;
	sign: (msg: string) => Promise<string>;
	disconnect: () => void;
}

const WalletCtx = createContext<WalletContextValue | null>(null);

export function WalletProvider({ children }: { children: ReactNode }) {
	const [state, setState] = useState<WalletState>({
		detected: false,
		connected: false,
		address: null,
		network: null,
		balance: null,
		loading: false,
		error: null,
	});

	// Detect KasWare on mount
	useEffect(() => {
		detectKasware().then((detected) => {
			setState((s) => ({ ...s, detected }));
		});
	}, []);

	// Subscribe to wallet events
	useEffect(() => {
		return subscribeToWallet((newState) => setState(newState));
	}, []);

	const connect = useCallback(async () => {
		if (state.connected) return;
		setState((s) => ({ ...s, loading: true, error: null }));
		try {
			const { address, network, balance } = await connectWallet();
			setState({
				detected: true,
				connected: true,
				address,
				network,
				balance,
				loading: false,
				error: null,
			});
		} catch (err) {
			setState((s) => ({
				...s,
				loading: false,
				error: (err as Error).message,
			}));
		}
	}, [state.connected]);

	const sign = useCallback(async (msg: string): Promise<string> => {
		return signMessage(msg, "schnorr");
	}, []);

	const disconnect = useCallback(() => {
		setState({
			detected: true,
			connected: false,
			address: null,
			network: null,
			balance: null,
			loading: false,
			error: null,
		});
	}, []);

	return (
		<WalletCtx.Provider value={{ state, connect, sign, disconnect }}>
			{children}
		</WalletCtx.Provider>
	);
}

export function useWallet(): WalletContextValue {
	const ctx = useContext(WalletCtx);
	if (!ctx) throw new Error("useWallet must be used within WalletProvider");
	return ctx;
}

/** Helper: returns the connected address or null. Shorthand. */
export function useAddress(): string | null {
	const { state } = useWallet();
	return state.connected ? state.address : null;
}
