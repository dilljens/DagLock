import { useWallet } from "../context/WalletContext";

export function ReputationPage() {
	const { state: wallet } = useWallet();
	return (
		<div>
			<div className="page-header">
				<h1>ReputationPage</h1>
				<p>Coming soon — connected as {wallet.address?.slice(0, 20)}…</p>
			</div>
		</div>
	);
}
