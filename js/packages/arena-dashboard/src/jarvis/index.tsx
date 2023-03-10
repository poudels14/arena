import { createSignal } from "solid-js";

const ChatBox = () => {
  const [expanded, setExpanded] = createSignal(false);
  return (
    <div class="rounded-lg border-4 border-slate-700">
      <div
        classList={{
          "block h-48 text-white rounded-t bg-brand-1": true,
          hidden: !expanded(),
        }}
      >
        <div class="p-2">show something here...</div>
      </div>
      <div>
        <input
          type="text"
          placeholder="Ask Arena..."
          class="font-medium px-3 py-2 w-full placeholder:text-gray-5 text-white bg-slate-700 outline-none"
          onFocus={() => setExpanded(true)}
          onBlur={() => setExpanded(false)}
        />
      </div>
    </div>
  );
};

export { ChatBox };
