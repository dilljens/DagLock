import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { HelmetProvider } from "react-helmet-async";
import userEvent from "@testing-library/user-event";
import { mockApi } from "./helpers";

function TestWrapper({ children }: { children: React.ReactNode }) {
	return <HelmetProvider>{children}</HelmetProvider>;
}

// Dynamic wallet mock — tests can override by setting mockWalletConnected
let mockWalletConnected = true;
vi.mock("../context/WalletContext", () => ({
	useWallet: () => ({
		state: {
			detected: true,
			connected: mockWalletConnected,
			address: mockWalletConnected ? "kaspa:qr6g5fsvq5h4c56j8w6q8w6q8w6q8w6q8w6q8w6q" : null,
			network: mockWalletConnected ? "testnet-12" : null,
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
		expect(screen.getByText("Generate Swap")).toBeInTheDocument();
		expect(screen.getByText("Submit Preimage")).toBeInTheDocument();
		expect(screen.getByText("How it Works")).toBeInTheDocument();
	});

	it("shows Generate tab by default with generate button", () => {
		r(<SwapPage />);
		expect(screen.getByRole("button", { name: /generate secret & hash/i })).toBeInTheDocument();
	});

	it("Generate tab creates secret and hash on click", async () => {
		const user = userEvent.setup();
		(api.generateSwap as ReturnType<typeof vi.fn>).mockResolvedValue({
			secret: "my_secret_hex",
			hash: "my_hash_hex",
		});

		r(<SwapPage />);

		const generateBtn = screen.getByRole("button", { name: /generate secret & hash/i });
		await user.click(generateBtn);

		await waitFor(() => {
			expect(screen.getByText("my_secret_hex")).toBeInTheDocument();
		});
		expect(screen.getByText("my_hash_hex")).toBeInTheDocument();
	});

	it("Generate tab shows error on API failure", async () => {
		const user = userEvent.setup();
		(api.generateSwap as ReturnType<typeof vi.fn>).mockRejectedValue(
			new Error("Generation failed"),
		);

		r(<SwapPage />);

		const generateBtn = screen.getByRole("button", { name: /generate secret & hash/i });
		await user.click(generateBtn);

		await waitFor(() => {
			expect(screen.getByText("Generation failed")).toBeInTheDocument();
		});
	});

	it("Submit Preimage tab renders form fields", async () => {
		const user = userEvent.setup();
		r(<SwapPage />);

		await user.click(screen.getByText("Submit Preimage"));

		expect(screen.getByPlaceholderText("esc_...")).toBeInTheDocument();
		expect(screen.getByPlaceholderText(/hex encoded secret from Generate tab/)).toBeInTheDocument();
	});

	it("Submit Preimage tab calls swapEscrow and shows success", async () => {
		const user = userEvent.setup();
		(api.swapEscrow as ReturnType<typeof vi.fn>).mockResolvedValue({
			status: "settled",
			escrow_id: "esc_1",
			method: "swap",
			preimage_hash: "abc123",
		});

		r(<SwapPage />);

		await user.click(screen.getByText("Submit Preimage"));

		const escrowInput = screen.getByPlaceholderText("esc_...");
		await user.type(escrowInput, "esc_1");

		const preimageInput = screen.getByPlaceholderText(/hex encoded secret from Generate tab/);
		await user.type(preimageInput, "deadbeef");

		const submitBtn = screen.getByRole("button", { name: /claim with preimage/i });
		await user.click(submitBtn);

		await waitFor(() => {
			expect(screen.getByText(/Preimage submitted!/)).toBeInTheDocument();
		});
	});

	it("Submit Preimage tab shows error on API failure", async () => {
		const user = userEvent.setup();
		(api.swapEscrow as ReturnType<typeof vi.fn>).mockRejectedValue(new Error("Invalid preimage"));

		r(<SwapPage />);

		await user.click(screen.getByText("Submit Preimage"));

		const escrowInput = screen.getByPlaceholderText("esc_...");
		await user.type(escrowInput, "esc_1");

		const preimageInput = screen.getByPlaceholderText(/hex encoded secret from Generate tab/);
		await user.type(preimageInput, "wrong");

		const submitBtn = screen.getByRole("button", { name: /claim with preimage/i });
		await user.click(submitBtn);

		await waitFor(() => {
			expect(screen.getByText("Invalid preimage")).toBeInTheDocument();
		});
	});

	it("How it Works tab shows protocol explanation", async () => {
		const user = userEvent.setup();
		r(<SwapPage />);

		await user.click(screen.getByText("How it Works"));

		expect(screen.getByText(/What is an Atomic Swap/)).toBeInTheDocument();
		expect(screen.getByText(/Step-by-Step/)).toBeInTheDocument();
		expect(screen.getByText(/Security Notes/)).toBeInTheDocument();
	});

	it("shows ConnectPrompt when wallet not connected on Submit tab", async () => {
		mockWalletConnected = false;
		const user = userEvent.setup();
		r(<SwapPage />);

		await user.click(screen.getByText("Submit Preimage"));

		expect(screen.getByText(/Connect KasWare to submit preimages/)).toBeInTheDocument();
		expect(screen.getByRole("button", { name: /connect wallet/i })).toBeInTheDocument();
	});
});
