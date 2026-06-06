/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
	plugins: [react()],
	resolve: {
		alias: {
			"@": path.resolve(__dirname, "src"),
		},
	},
	server: {
		port: 5173,
		proxy: {
			"/v1": {
				target: "http://localhost:8443",
				changeOrigin: true,
			},
		},
	},
	build: {
		outDir: "dist",
		sourcemap: true,
	},
	test: {
		environment: "jsdom",
		setupFiles: "./src/__tests__/setup.ts",
		globals: true,
	},
});
