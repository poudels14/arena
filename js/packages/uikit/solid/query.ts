import { createSignal } from "solid-js";

type Query<Args extends Array<unknown>> = (...args: Args) => Promise<unknown>;

type MutationQuery<Args extends Array<unknown>> = {
  (...args: Args): void;
  error?: any;
  success?: boolean;
};

type MutationQueryOptions = {
  onError: (e: any) => void;
};

function createMutationQuery<A1>(
  q: Query<[A1]>,
  options?: MutationQueryOptions
): MutationQuery<[A1]>;
function createMutationQuery<A1>(
  query: Query<[A1]>,
  options?: MutationQueryOptions
) {
  const [success, setSucceess] = createSignal<boolean | undefined>(undefined);
  const [error, setError] = createSignal<any | undefined>(undefined);

  return Object.defineProperties(
    (...args: Parameters<typeof query>) => {
      query(...args)
        .then((r) => {
          console.log(r);
          setSucceess(true);
        })
        .catch((e) => {
          console.log("ERROR =", e);
          setError(e);
          options?.onError?.(e);
        });
    },
    {
      error: {
        get: error,
      },
      success: {
        get: success,
      },
    }
  );
}

export { createMutationQuery };
