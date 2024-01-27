import { Show, Switch, Match, createSignal } from "solid-js";

function Login() {
  const [state, setState] = createSignal({
    email: undefined,
    loginCode: undefined,
  });
  return (
    <main class="text-center mx-auto text-gray-700 p-4">
      <div class="flex mt-24 justify-center">
        <div class="flex flex-col space-y-3">
          <div class="font-medium text-xl">Sign in to Sidecar</div>
          <div class="min-w-[350px] px-8 py-8 rounded-md border border-gray-200 shadow-md text-left space-y-2">
            <div class="">
              <label class="space-y-1.5">
                <div class="text-sm font-medium">Email</div>
                <input
                  type="email"
                  name="email"
                  placeholder="email@gmail.com"
                  class="w-full px-2 py-1.5 text-sm border border-gray-200 bg2-gray-200 rounded outline-none focus:ring-1"
                />
              </label>
            </div>
            <Show when={state().email}>
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
            </Show>
            <div>
              <button
                type="button"
                class="w-full py-1 text-sm text-white bg-indigo-500 text-center rounded"
              >
                <Switch>
                  <Match when={!state().email}>Sign In</Match>
                  <Match when={state().email}>Cotinue</Match>
                </Switch>
              </button>
            </div>
          </div>
        </div>
      </div>
    </main>
  );
}

export default Login;
