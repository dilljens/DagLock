/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { VitePWA } from "vite-plugin-pwa";
import path from "path";

export default defineConfig({
	plugins: [
		react(),
		VitePWA({
			registerType: "autoUpdate",
			manifest: {
				name: "DagLock",
				short_name: "DagLock",
				description: "Trustless escrow and atomic swaps on Kaspa",
				theme_color: "#0a1a0a",
				background_color: "#0a1a0a",
				display: "standalone",
				icons: [
					{ src: "/icon-192.png", sizes: "192x192", type: "image/png" },
					{ src: "/icon-512.png", sizes: "512x512", type: "image/png" },
				],
			},
		}),
	],
	resolve: {
		alias: {
			"@": path.resolve(__dirname, "src"),
		},
	},
	server: {
		port: 5174,
		proxy: {
			"/v1": {
				target: "http://localhost:8443",
				changeOrigin: true,
			},
		},
	},
	build: {
		outDir: "dist",
		sourcemap: false,
	},
	test: {
		environment: "jsdom",
		setupFiles: "./src/__tests__/setup.ts",
		globals: true,
		exclude: ["e2e/**", "node_modules/**"],
	},
});
