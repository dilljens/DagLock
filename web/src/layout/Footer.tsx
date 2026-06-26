import { useWallet } from "../context/WalletContext";

export function Footer() {
	const { state } = useWallet();
	return (
		<footer
			style={{
				marginTop: "48px",
				padding: "24px 0",
				borderTop: "1px solid var(--color-border)",
				fontSize: "13px",
				color: "var(--color-text-muted)",
			}}
		>
			<div
				style={{
					display: "flex",
					flexWrap: "wrap",
					gap: "16px 24px",
					justifyContent: "space-between",
					alignItems: "center",
				}}
			>
				<div style={{ display: "flex", gap: "16px", flexWrap: "wrap" }}>
					<span>DagLock v0.1.0</span>
					<span>·</span>
					<span>0.5% escrow fee · 0.1% vault fee</span>
					<span>·</span>
					<a
						href="https://github.com/dilljens/DagLock"
						target="_blank"
						rel="noopener noreferrer"
						style={{ color: "var(--color-text-secondary)", textDecoration: "underline" }}
					>
						GitHub
					</a>
					<span>·</span>
					<a
						href="https://t.me/DagLock_bot"
						target="_blank"
						rel="noopener noreferrer"
						style={{ color: "var(--color-text-secondary)", textDecoration: "underline" }}
					>
						@DagLock_bot
					</a>
					<span>·</span>
					<a
						href="https://github.com/dilljens/DagLock/issues"
						target="_blank"
						rel="noopener noreferrer"
						style={{ color: "var(--color-text-secondary)", textDecoration: "underline" }}
					>
						Report Bug
					</a>
				</div>
				<div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
					{state.network && (
						<span
							className="pill"
							style={{
								background:
									state.network === "mainnet" ? "rgba(83,215,105,0.15)" : "rgba(255,152,0,0.15)",
								color: state.network === "mainnet" ? "#53d769" : "#ff9800",
								fontSize: "11px",
							}}
						>
							{state.network}
						</span>
					)}
				</div>
			</div>
			<div style={{ marginTop: "8px", lineHeight: 1.5 }}>
				Funds locked in SilverScript covenants — no admin keys, no backdoors. Fees: 0.5% on escrow
				settlement, 0.1% on vault withdrawal. Both enforced by the covenant itself and paid to the
				treasury.
			</div>
		</footer>
	);
}
