import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { HelmetProvider } from "react-helmet-async";
import { SecurityPage } from "../pages/SecurityPage";

function renderPage() {
	return render(
		<HelmetProvider>
			<SecurityPage />
		</HelmetProvider>,
	);
}

describe("SecurityPage", () => {
	it("renders the page title", () => {
		renderPage();
		expect(screen.getByText((c) => c.includes("Covenant Security Analysis"))).toBeInTheDocument();
	});

	it("renders all 6 attack scenarios", () => {
		renderPage();
		expect(screen.getByText("Arbiter tries to steal")).toBeInTheDocument();
		expect(screen.getByText("Server changes the fee")).toBeInTheDocument();
		expect(screen.getByText("Seller ships nothing")).toBeInTheDocument();
		expect(screen.getByText("Buyer ghosts after receiving")).toBeInTheDocument();
		expect(screen.getByText("Arbiter disappears")).toBeInTheDocument();
		expect(screen.getByText("Chat evidence forged")).toBeInTheDocument();
	});

	it("shows Execute attack buttons for each scenario", () => {
		renderPage();
		const buttons = screen.getAllByText("Execute attack");
		expect(buttons).toHaveLength(6);
	});

	it("shows attack result when button is clicked", () => {
		renderPage();
		const buttons = screen.getAllByText("Execute attack");
		fireEvent.click(buttons[0]);
		expect(screen.getByText(/Attack Failed/)).toBeInTheDocument();
		expect(screen.getByText("Reset")).toBeInTheDocument();
	});

	it("shows the stats bar", () => {
		renderPage();
		expect(screen.getByText("Attack scenarios")).toBeInTheDocument();
		expect(screen.getByText("Attacks succeeded")).toBeInTheDocument();
		expect(screen.getByText("Covenant block rate")).toBeInTheDocument();
	});

	it("allows resetting after an attack", () => {
		renderPage();
		const buttons = screen.getAllByText("Execute attack");
		fireEvent.click(buttons[0]);
		const reset = screen.getByText("Reset");
		fireEvent.click(reset);
		expect(screen.queryByText("Reset")).not.toBeInTheDocument();
	});
});
