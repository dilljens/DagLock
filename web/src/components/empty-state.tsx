import type { ReactNode } from "react";

interface EmptyStateProps {
	icon: string;
	title: string;
	description: string;
	action?: {
		label: string;
		onClick: () => void;
	};
	children?: ReactNode;
}

export function EmptyState({ icon, title, description, action, children }: EmptyStateProps) {
	return (
		<div className="empty-state">
			<div className="empty-state-icon">{icon}</div>
			<h3>{title}</h3>
			<p>{description}</p>
			{action && (
				<button className="button primary" type="button" onClick={action.onClick}>
					{action.label}
				</button>
			)}
			{children}
		</div>
	);
}
