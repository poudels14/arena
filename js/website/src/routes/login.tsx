import { Show, createSignal } from "solid-js";

function Login() {
  const [state, setState] = createSignal<any>({
    email: undefined,
    emailSent: false,
    error: null,
  });

  const sendMagicLink = () => {
    fetch("/api/account/login/magic/send", {
      method: "POST",
      body: JSON.stringify({
        email: state().email,
      }),
      headers: {
        "Content-type": "application/json",
      },
    }).then(async (res) => {
      if (!res.ok) {
        const error = await res.text();
        setState((prev) => {
          return {
            ...prev,
            error,
          };
        });
        return;
      }
      setState((prev) => {
        return {
          ...prev,
          emailSent: true,
        };
      });
    });
  };
  return (
    <main class="text-center mx-auto text-gray-700 p-4">
      <div class="flex mt-24 justify-center">
        <div class="flex flex-col space-y-3">
          <div class="font-medium text-xl">Sign in to Sidecar</div>
          <div class="w-[350px] px-8 py-8 rounded-md border border-gray-200 shadow-md text-left space-y-2">
            <Show when={state().error}>
              <div class="text-xs text-center text-red-600">
                <div class="line-clamp-3">{state().error}</div>
                <div>Please try again</div>
              </div>
            </Show>
            <form
              class=""
              onSubmit={(e) => {
                e.preventDefault();
                sendMagicLink();
              }}
            >
              <label class="space-y-1.5">
                <div class="text-sm font-medium">Email</div>
                <input
                  type="email"
                  name="email"
                  placeholder="email@gmail.com"
                  class="w-full px-2 py-1.5 text-sm border border-gray-200 bg2-gray-200 rounded outline-none focus:ring-1"
                  onInput={(e) =>
                    setState((prev) => {
                      return {
                        ...prev,
                        error: null,
                        emailSent: false,
                        email: e.target.value,
                      };
                    })
                  }
                />
              </label>
            </form>
            {/* <Show when={state().email}>
              <div class="px-8 text-sm text-center text-gray-400">
                <div>A temporary login code has been sent to you email</div>
                <div>Please check your inbox</div>
              </div>
              <div>
                <label class="space-y-3">
                  <div class="text-sm font-medium">Login code</div>
                  <input
                    type="text"
                    name="code"
                    placeholder="Paste login code"
                    class="w-full px-2 py-1.5 text-sm border border-gray-200 bg2-gray-200 rounded outline-none focus:ring-1"
                  />
                </label>
              </div>
            </Show> */}
            <Show when={state().emailSent}>
              <div class="font-semibold text-center text-gray-600">
                <div>Check your email for a login link</div>
              </div>
            </Show>
            <Show when={!state().emailSent}>
              <div class="pt-5">
                <button
                  type="button"
                  class="w-full py-1 text-sm text-white bg-indigo-500 text-center rounded"
                  onClick={() => {
                    sendMagicLink();
                  }}
                >
                  Sign In
                </button>
              </div>
            </Show>
          </div>
        </div>
      </div>
    </main>
  );
}

export default Login;
