import { Show, createSignal } from "solid-js";

type State = {
  firstName: string;
  lastName: string;
  email: string;
  message: string;
  accountCreated: boolean;
  error: string | null;
};

function Waitlist() {
  const [getState, setState] = createSignal<State>({
    firstName: "",
    lastName: "",
    email: "",
    message: "",
    accountCreated: false,
    error: null,
  });

  const joinWaitlist = () => {
    const state = getState();
    fetch("/api/account/signup", {
      method: "POST",
      body: JSON.stringify({
        firstName: state.firstName,
        lastName: state.lastName,
        email: state.email,
        message: state.message,
      }),
      headers: {
        "Content-type": "application/json",
      },
    }).then(async (res) => {
      if (!res.ok) {
        const response = await res.json();
        setState((prev) => {
          return {
            ...prev,
            error: response.error,
          };
        });
        return;
      }
      setState((prev) => {
        return {
          ...prev,
          error: null,
          accountCreated: true,
        };
      });
    });
  };
  return (
    <main class="text-center mx-auto text-gray-700 p-4">
      <div class="flex mt-16 justify-center">
        <div class="flex flex-col space-y-3">
          <div class="font-medium text-xl">Sign up for Portal Cloud</div>
          <div class="w-[350px] px-8 pt-4 pb-8 rounded-md border border-gray-200 shadow-md text-left space-y-2">
            <div class="h-4 text-xs text-center text-red-600">
              <Show when={getState().error}>
                <div class="line-clamp-3">{getState().error}</div>
              </Show>
            </div>

            <Show when={!getState().accountCreated}>
              <form
                method="post"
                class="space-y-4"
                action="/waitlist"
                onSubmit={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  joinWaitlist();
                }}
              >
                <Input
                  title="First name"
                  name="firstName"
                  placeholder="John"
                  type="text"
                  onInput={(firstName) =>
                    setState((prev) => {
                      return {
                        ...prev,
                        firstName,
                      };
                    })
                  }
                />
                <Input
                  title="Last name"
                  name="lastName"
                  placeholder="Doe"
                  type="text"
                  onInput={(lastName) =>
                    setState((prev) => {
                      return {
                        ...prev,
                        lastName,
                      };
                    })
                  }
                />
                <Input
                  title="Email"
                  name="email"
                  placeholder="john@example.com"
                  type="email"
                  onInput={(email) =>
                    setState((prev) => {
                      return {
                        ...prev,
                        email,
                      };
                    })
                  }
                />
                <div>
                  <label class="text-sm space-y-1.5">
                    <div class="font-medium">Message</div>
                    <textarea
                      class="w-full h-16 px-2 py-1.5 outline-none rounded border border-gray-200 focus:ring-1"
                      placeholder="Tell us about your use case"
                      onInput={(e) => {
                        setState((prev) => {
                          return {
                            ...prev,
                            message: e.target.value,
                          };
                        });
                      }}
                    />
                  </label>
                </div>
                <div class="pt-3">
                  <button
                    type="button"
                    class="w-full py-1 text-sm text-white bg-indigo-500 text-center rounded"
                    onClick={() => {
                      joinWaitlist();
                    }}
                  >
                    Join waitlist
                  </button>
                </div>
              </form>
            </Show>
            <Show when={getState().accountCreated}>
              <div class="font-semibold text-center text-gray-600">
                <div>Thanks for joining the waitlist</div>
              </div>
            </Show>
          </div>
        </div>
      </div>
    </main>
  );
}

const Input = (props: {
  title: string;
  name: string;
  type: string;
  placeholder: string;
  onInput: (value: string) => void;
}) => {
  return (
    <div>
      <label class="text-sm space-y-1.5">
        <div class="font-medium">{props.title}</div>
        <input
          type={props.type}
          name={props.name}
          placeholder={props.placeholder}
          class="w-full px-2 py-1.5 border border-gray-200 rounded outline-none focus:ring-1"
          onInput={(e) => props.onInput(e.target.value)}
        />
      </label>
    </div>
  );
};

export default Waitlist;
