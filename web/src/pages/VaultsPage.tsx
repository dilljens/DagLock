import { useWallet } from "../context/WalletContext";

export function VaultsPage() {
	const { state: wallet } = useWallet();
	return (
		<div>
			<div className="page-header">
				<h1>VaultsPage</h1>
				<p>Coming soon — connected as {wallet.address?.slice(0, 20)}…</p>
			</div>
		</div>
	);
}
