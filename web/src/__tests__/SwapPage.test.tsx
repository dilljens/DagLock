import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { HelmetProvider } from "react-helmet-async";
import userEvent from "@testing-library/user-event";
import { mockApi } from "./helpers";

function TestWrapper({ children }: { children: React.ReactNode }) {
	return <HelmetProvider>{children}</HelmetProvider>;
}

// Dynamic wallet mock
let mockWalletConnected = true;
vi.mock("../context/WalletContext", () => ({
	useWallet: () => ({
		state: {
			detected: true,
			connected: mockWalletConnected,
			address: mockWalletConnected ? "kaspa:qr6g5fsvq5h4c56j8w6q8w6q8w6q8w6q8w6q8w6q" : null,
			network: mockWalletConnected ? "testnet-10" : null,
			balance: mockWalletConnected ? 1000 : null,
			loading: false,
			error: null,
		},
		connect: vi.fn(),
		sign: vi.fn().mockResolvedValue("ab".repeat(32)),
		disconnect: vi.fn(),
	}),
	useAddress: () => (mockWalletConnected ? "kaspa:qr6g5fsvq5h4c56j8w6q8w6q8w6q8w6q8w6q8w6q" : null),
}));
vi.mock("../layout/Toast", () => ({
	useToast: () => ({ notify: vi.fn() }),
}));
vi.mock("../api", () => ({ api: mockApi() }));

import { api } from "../api";
import { SwapPage } from "../pages/SwapPage";

const r = (ui: React.ReactElement) => render(ui, { wrapper: TestWrapper });

describe("SwapPage", () => {
	beforeEach(() => {
		vi.clearAllMocks();
		mockWalletConnected = true;
	});

	it("renders all three tabs", () => {
		r(<SwapPage />);
		expect(screen.getByText("Create Swap")).toBeInTheDocument();
		expect(screen.getByText("Claim Swap")).toBeInTheDocument();
		expect(screen.getByText("How it Works")).toBeInTheDocument();
	});

	it("shows Create tab by default with terms form", () => {
		r(<SwapPage />);
		expect(screen.getByText(/Start an Atomic Swap/)).toBeInTheDocument();
		expect(screen.getByPlaceholderText("100")).toBeInTheDocument();
	});

	it("Swap wizard generates secret", async () => {
		const user = userEvent.setup();
		(api.generateSwap as ReturnType<typeof vi.fn>).mockResolvedValue({
			secret: "my_secret_hex",
			hash: "my_hash_hex",
		});

		r(<SwapPage />);

		// Fill in the terms form
		const amountInput = screen.getByPlaceholderText("100");
		await user.type(amountInput, "100");

		const counterpartyInput = screen.getByPlaceholderText("kaspa:...");
		await user.type(counterpartyInput, "kaspa:test1234567890");

		await user.click(screen.getByRole("button", { name: /next: generate secret/i }));

		// Should see the generate secret screen — use getAllByText since step indicator also shows it
		const secretHeadings = screen.getAllByText(/Generate Secret/);
		expect(secretHeadings.length).toBeGreaterThanOrEqual(1);

		// Check the safety checkbox first
		await user.click(screen.getByRole("checkbox"));

		// Click generate secret
		await user.click(screen.getByRole("button", { name: /generate secret & continue/i }));

		await waitFor(() => {
			expect(screen.getByText("my_secret_hex")).toBeInTheDocument();
		});
	});

	it("Swap wizard handles API failure", async () => {
		const user = userEvent.setup();
		(api.generateSwap as ReturnType<typeof vi.fn>).mockRejectedValue(
			new Error("Generation failed"),
		);

		r(<SwapPage />);

		const amountInput = screen.getByPlaceholderText("100");
		await user.type(amountInput, "100");

		const counterpartyInput = screen.getByPlaceholderText("kaspa:...");
		await user.type(counterpartyInput, "kaspa:test1234567890");

		await user.click(screen.getByRole("button", { name: /next: generate secret/i }));

		// Check the safety checkbox first
		await user.click(screen.getByRole("checkbox"));

		await user.click(screen.getByRole("button", { name: /generate secret & continue/i }));

		await waitFor(() => {
			expect(screen.getByText("Generation failed")).toBeInTheDocument();
		});
	});

	it("Claim tab renders escrow search", async () => {
		const user = userEvent.setup();
		r(<SwapPage />);

		await user.click(screen.getByText("Claim Swap"));

		expect(screen.getByPlaceholderText(/Escrow ID/)).toBeInTheDocument();
		expect(screen.getByRole("button", { name: /fetch escrow/i })).toBeInTheDocument();
	});

	it("How it Works tab shows explanation", async () => {
		const user = userEvent.setup();
		r(<SwapPage />);

		await user.click(screen.getByText("How it Works"));

		expect(screen.getByText(/What is an Atomic Swap/)).toBeInTheDocument();
		expect(screen.getByText(/How the Wizard Works/)).toBeInTheDocument();
		expect(screen.getByText(/Security Notes/)).toBeInTheDocument();
	});

	it("shows ConnectPrompt when wallet not connected on Claim tab", async () => {
		mockWalletConnected = false;
		const user = userEvent.setup();
		r(<SwapPage />);

		await user.click(screen.getByText("Claim Swap"));

		expect(screen.getByText(/Connect your wallet/)).toBeInTheDocument();
		expect(screen.getByRole("button", { name: /connect wallet/i })).toBeInTheDocument();
	});
});
