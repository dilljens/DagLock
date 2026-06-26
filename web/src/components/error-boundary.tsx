import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
	children: ReactNode;
	fallback?: ReactNode;
}

interface State {
	error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
	state: State = { error: null };

	static getDerivedStateFromError(error: Error): State {
		return { error };
	}

	componentDidCatch(error: Error, info: ErrorInfo) {
		console.error("ErrorBoundary caught:", error, info.componentStack);
	}

	render() {
		if (this.state.error) {
			if (this.props.fallback) return this.props.fallback;
			return (
				<div style={{ padding: "32px", textAlign: "center" }}>
					<div style={{ fontSize: "2rem", marginBottom: "12px" }}>⚠</div>
					<h3 style={{ color: "#ff7b7b", margin: "0 0 8px" }}>Something went wrong</h3>
					<p style={{ color: "#88b888", fontSize: "14px", margin: "0 0 16px" }}>
						{this.state.error.message}
					</p>
					<button className="button primary" onClick={() => this.setState({ error: null })}>
						Try Again
					</button>
				</div>
			);
		}
		return this.props.children;
	}
}
