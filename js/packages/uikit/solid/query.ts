import { createSignal } from "solid-js";

type MutationQuery<Query> = Query & {
  response: any;
  error?: any;
  success?: boolean;
};

type MutationQueryOptions = {
  onError: (e: any) => void;
};

function createMutationQuery<Query extends (...args: any[]) => Promise<any>>(
  q: Query,
  options?: MutationQueryOptions
): MutationQuery<Query>;
function createMutationQuery<Query extends (...args: any[]) => Promise<any>>(
  query: Query,
  options?: MutationQueryOptions
) {
  const [response, setResponse] = createSignal<unknown>(null);
  const [success, setSucceess] = createSignal<boolean | undefined>(undefined);
  const [error, setError] = createSignal<any | undefined>(undefined);

  return Object.defineProperties(
    (...args: Parameters<typeof query>) => {
      query(...args)
        .then((r: any) => {
          setResponse(r);
          setSucceess(true);
        })
        .catch((e: any) => {
          setError(e);
          options?.onError?.(e);
        });
    },
    {
      response: { get: response },
      error: {
        get: error,
      },
      success: {
        get: success,
      },
    }
  );
}

export type { MutationQuery };
export { createMutationQuery };
