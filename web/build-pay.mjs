// Standalone build for <daglock-pay> web component
import * as esbuild from "esbuild";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const result = await esbuild.build({
	entryPoints: [path.join(__dirname, "src/components/daglock-pay.ts")],
	outfile: path.join(__dirname, "dist/daglock-pay.js"),
	bundle: true,
	minify: true,
	sourcemap: false,
	target: "es2020",
	format: "iife",
	globalName: "DaglockPay",
	define: {
		"window.__DAGLOCK_API_BASE__": '""',
		"window.__DAGLOCK_URL__": '"https://daglock.com"',
	},
	loader: {
		".ts": "ts",
	},
});

console.log(
	`Built daglock-pay.js (${(result.outputFiles?.[0]?.text?.length ?? "?").toLocaleString()} bytes)`,
);
