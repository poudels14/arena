import { createSignal } from "solid-js";

const ChatBox = () => {
  const [expanded, setExpanded] = createSignal(false);
  return (
    <div class="rounded-lg border-4 border-brand-12/90">
      <div
        classList={{
          "block h-48 text-brand-1 rounded-t bg-brand-1": true,
          hidden: !expanded(),
        }}
      >
        <div class="p-2">show something here...</div>
      </div>
      <div>
        <input
          type="text"
          placeholder="Ask Arena..."
          class="font-medium px-3 py-2 w-full placeholder:text-brand-9 text-brand-1 bg-brand-12/90 backdrop-blur outline-none"
          onFocus={() => setExpanded(true)}
          onBlur={() => setExpanded(false)}
        />
      </div>
    </div>
  );
};

export { ChatBox };
