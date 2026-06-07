import { useWallet } from "../context/WalletContext";

export function JuryPage() {
	const { state: wallet } = useWallet();
	return (
		<div>
			<div className="page-header">
				<h1>JuryPage</h1>
				<p>Coming soon — connected as {wallet.address?.slice(0, 20)}…</p>
			</div>
		</div>
	);
}
