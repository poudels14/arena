import { mount, ClientRoot } from "@portal/solidjs/client";
import { QueryClientProvider } from "@portal/solid-query";
import * as Sentry from "@sentry/browser";
import Root from "./app/root";

Sentry.init({
  dsn: "https://b6d70976adbb0932725f7b4817e422cf@o4507128581914624.ingest.us.sentry.io/4507131496366080",
  integrations: [],
  tracesSampleRate: 1.0,
  profilesSampleRate: 0.0,
});

const globalFetch = fetch;
globalThis.fetch = async () => {
  throw new Error("`fetch` not supported. Use `@portal/solid-query` instead");
};

const RootWithQueryClient = () => {
  return (
    <QueryClientProvider client={globalFetch}>
      <Root />
    </QueryClientProvider>
  );
};

mount(() => <ClientRoot Root={RootWithQueryClient} />, document);
