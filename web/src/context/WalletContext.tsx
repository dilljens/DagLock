// Wallet context for DagLock — provides connected wallet state to all components.
// Eliminates the need for manual "your address" inputs everywhere.

import { createContext, useContext, useState, useEffect, useCallback, type ReactNode } from "react";
import {
	detectKasware,
	connectWallet,
	signMessage,
	mockSignature,
	subscribeToWallet,
	type WalletState,
} from "../kasware";

export interface WalletContextValue {
	state: WalletState;
	connect: () => Promise<void>;
	sign: (msg: string) => Promise<string>;
	disconnect: () => void;
	/** Set a manual address for testnet dev mode (no wallet needed). */
	setManualAddress: (address: string) => void;
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
		manualMode: false,
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
				manualMode: false,
			});
		} catch (err) {
			setState((s) => ({
				...s,
				loading: false,
				error: (err as Error).message,
			}));
		}
	}, [state.connected]);

	const setManualAddress = useCallback((address: string) => {
		setState({
			detected: false,
			connected: true,
			address,
			network: "testnet-12",
			balance: null,
			loading: false,
			error: null,
			manualMode: true,
		});
	}, []);

	const sign = useCallback(
		async (msg: string): Promise<string> => {
			if (state.manualMode) {
				return mockSignature(msg);
			}
			return signMessage(msg, "schnorr");
		},
		[state.manualMode],
	);

	const disconnect = useCallback(() => {
		setState({
			detected: !state.manualMode,
			connected: false,
			address: null,
			network: null,
			balance: null,
			loading: false,
			error: null,
			manualMode: false,
		});
	}, [state.manualMode]);

	return (
		<WalletCtx.Provider value={{ state, connect, sign, disconnect, setManualAddress }}>
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
