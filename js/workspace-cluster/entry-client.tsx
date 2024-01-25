import { mount, ClientRoot } from "@portal/solidjs/client";
import { QueryClientProvider } from "@portal/solid-query";
import Root from "@portal/workspace/app/root";

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
