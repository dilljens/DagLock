import { useEffect, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";

const WS_URL = import.meta.env.VITE_WS_URL || "wss://api.daglock.com/v1/ws";
const RECONNECT_DELAY = 5000;

/**
 * Map WebSocket events to query keys that should be invalidated.
 * This keeps the UI in sync without polling.
 */
const EVENT_TO_QUERIES: Record<string, string[]> = {
	escrow_created: ["escrows", "stats", "offers"],
	escrow_settled: ["escrows", "stats"],
	escrow_refunded: ["escrows", "stats"],
	escrow_cancelled: ["escrows", "stats"],
	escrow_disputed: ["escrows", "stats", "jury"],
	escrow_expired: ["escrows", "stats"],
	offer_created: ["offers"],
	offer_accepted: ["offers", "escrows"],
	offer_cancelled: ["offers"],
};

export function useWebSocket() {
	const queryClient = useQueryClient();
	const wsRef = useRef<WebSocket | null>(null);
	const reconnectTimer = useRef<ReturnType<typeof setTimeout>>();

	useEffect(() => {
		function connect() {
			if (wsRef.current?.readyState === WebSocket.OPEN) return;

			const ws = new WebSocket(WS_URL);
			wsRef.current = ws;

			ws.onopen = () => {
				console.debug("[WS] Connected");
			};

			ws.onmessage = (event) => {
				try {
					const msg = JSON.parse(event.data);
					const queries = EVENT_TO_QUERIES[msg.event];
					if (queries) {
						for (const key of queries) {
							queryClient.invalidateQueries({ queryKey: [key] });
						}
					}
				} catch {
					// Ignore malformed messages
				}
			};

			ws.onclose = () => {
				console.debug("[WS] Disconnected, reconnecting in 5s");
				reconnectTimer.current = setTimeout(connect, RECONNECT_DELAY);
			};

			ws.onerror = () => {
				ws.close();
			};
		}

		connect();

		return () => {
			clearTimeout(reconnectTimer.current);
			if (wsRef.current) {
				wsRef.current.onclose = null; // prevent reconnect on unmount
				wsRef.current.close();
			}
		};
	}, [queryClient]);
}
