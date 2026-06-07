// Simple hash-based router for DagLock
// No external dependencies. Uses location.hash and popstate events.

import { useState, useEffect, useCallback, createContext, useContext, type ReactNode } from "react";

export type Route =
	| "/"
	| "/offers"
	| "/escrows"
	| "/vaults"
	| "/reputation"
	| "/jury"
	| "/settings";

interface RouterContextValue {
	route: Route;
	navigate: (to: Route) => void;
}

const RouterContext = createContext<RouterContextValue | null>(null);

/** Normalize a hash string to a valid Route */
function hashToRoute(hash: string): Route {
	const path = hash.replace(/^#/, "").split("?")[0].replace(/\/$/, "") || "/";
	const validRoutes: Route[] = [
		"/",
		"/offers",
		"/escrows",
		"/vaults",
		"/reputation",
		"/jury",
		"/settings",
	];
	// Support redirects from old-style anchors like #offers → /offers
	if (path.startsWith("#")) return hashToRoute(path);
	if (validRoutes.includes(path as Route)) return path as Route;
	return "/";
}

export function RouterProvider({ children }: { children: ReactNode }) {
	const [route, setRoute] = useState<Route>(() => hashToRoute(location.hash));

	useEffect(() => {
		const onHashChange = () => setRoute(hashToRoute(location.hash));
		window.addEventListener("hashchange", onHashChange);
		return () => window.removeEventListener("hashchange", onHashChange);
	}, []);

	const navigate = useCallback((to: Route) => {
		location.hash = to;
	}, []);

	return <RouterContext.Provider value={{ route, navigate }}>{children}</RouterContext.Provider>;
}

export function useRouter(): RouterContextValue {
	const ctx = useContext(RouterContext);
	if (!ctx) throw new Error("useRouter must be used within RouterProvider");
	return ctx;
}
