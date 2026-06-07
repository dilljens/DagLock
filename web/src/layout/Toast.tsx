import { createContext, useContext, useState, useCallback, type ReactNode } from "react";

export type ToastType = "success" | "error" | "info";

export interface Toast {
	id: number;
	type: ToastType;
	title: string;
	message?: string;
}

interface ToastContextValue {
	toasts: Toast[];
	notify: (type: ToastType, title: string, message?: string) => void;
	dismiss: (id: number) => void;
}

const ToastCtx = createContext<ToastContextValue | null>(null);
let nextId = 0;

export function ToastProvider({ children }: { children: ReactNode }) {
	const [toasts, setToasts] = useState<Toast[]>([]);

	const dismiss = useCallback((id: number) => {
		setToasts((prev) => prev.filter((t) => t.id !== id));
	}, []);

	const notify = useCallback(
		(type: ToastType, title: string, message?: string) => {
			const id = nextId++;
			setToasts((prev) => [...prev, { id, type, title, message }]);
			setTimeout(() => dismiss(id), 5000);
		},
		[dismiss],
	);

	return (
		<ToastCtx.Provider value={{ toasts, notify, dismiss }}>
			{children}
			<div className="toast-container">
				{toasts.map((t) => (
					<div key={t.id} className={`toast toast--${t.type}`}>
						<div className="toast-body">
							<div className="toast-title">{t.title}</div>
							{t.message && <div className="toast-message">{t.message}</div>}
						</div>
						<button className="toast-close" onClick={() => dismiss(t.id)}>
							✕
						</button>
					</div>
				))}
			</div>
		</ToastCtx.Provider>
	);
}

export function useToast(): ToastContextValue {
	const ctx = useContext(ToastCtx);
	if (!ctx) throw new Error("useToast must be used within ToastProvider");
	return ctx;
}
