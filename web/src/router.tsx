// Simple History API-based router for DagLock
// Uses pushState/popstate so routes are real URLs indexable by Google.
// No external dependencies.

import { useState, useEffect, useCallback, createContext, useContext, type ReactNode } from "react";

export type Route =
	| "/"
	| "/offers"
	| "/escrows"
	| "/swap"
	| "/vaults"
	| "/reputation"
	| "/jury"
	| "/docs";

interface RouterContextValue {
	route: Route;
	navigate: (to: Route) => void;
}

const RouterContext = createContext<RouterContextValue | null>(null);

const VALID_ROUTES: readonly Route[] = [
	"/",
	"/offers",
	"/escrows",
	"/vaults",
	"/reputation",
	"/jury",
	"/swap",
	"/docs",
];

/** Read the current pathname and normalize to a valid Route */
function pathToRoute(path: string): Route {
	const clean = path.replace(/\/$/, "") || "/";
	if (VALID_ROUTES.includes(clean as Route)) return clean as Route;
	return "/";
}

export function RouterProvider({ children }: { children: ReactNode }) {
	const [route, setRoute] = useState<Route>(() => pathToRoute(window.location.pathname));

	useEffect(() => {
		// Handle browser back/forward
		const onPop = () => setRoute(pathToRoute(window.location.pathname));
		window.addEventListener("popstate", onPop);
		return () => window.removeEventListener("popstate", onPop);
	}, []);

	const navigate = useCallback((to: Route) => {
		if (to === window.location.pathname) return; // skip if already there
		window.history.pushState(null, "", to);
		setRoute(to);
		window.scrollTo(0, 0);
	}, []);

	return <RouterContext.Provider value={{ route, navigate }}>{children}</RouterContext.Provider>;
}

export function useRouter(): RouterContextValue {
	const ctx = useContext(RouterContext);
	if (!ctx) throw new Error("useRouter must be used within RouterProvider");
	return ctx;
}
