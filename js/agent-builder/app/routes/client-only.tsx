import { useSyncExternalStore } from "react";

function subscribe() {
  return () => {};
}

export function useHydrated() {
  return useSyncExternalStore(
    subscribe,
    () => true,
    () => false
  );
}

export function ClientOnly({ children, fallback = null }: any) {
  return useHydrated() ? <>{children()}</> : <>{fallback}</>;
}
