import { createContext, useContext, useState, useCallback, type ReactNode } from "react";
import { motion, AnimatePresence } from "motion/react";

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

const ICONS: Record<ToastType, string> = {
	success: "✓",
	error: "✕",
	info: "ℹ",
};

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
				<AnimatePresence>
					{toasts.map((t) => (
						<motion.div
							key={t.id}
							initial={{ opacity: 0, scale: 0.8, y: -20 }}
							animate={{ opacity: 1, scale: 1, y: 0 }}
							exit={{ opacity: 0, scale: 0.8, y: -10 }}
							transition={{ duration: 0.25, ease: [0.16, 1, 0.3, 1] }}
							className={`toast toast--${t.type}`}
						>
							<span className={`toast-icon toast-icon--${t.type}`}>{ICONS[t.type]}</span>
							<div className="toast-body">
								<div className="toast-title">{t.title}</div>
								{t.message && <div className="toast-message">{t.message}</div>}
							</div>
							<button className="toast-close" onClick={() => dismiss(t.id)}>
								✕
							</button>
						</motion.div>
					))}
				</AnimatePresence>
			</div>
		</ToastCtx.Provider>
	);
}

export function useToast(): ToastContextValue {
	const ctx = useContext(ToastCtx);
	if (!ctx) throw new Error("useToast must be used within ToastProvider");
	return ctx;
}
