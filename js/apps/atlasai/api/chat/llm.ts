import type { OpenAIChat } from "./OpenAI";

/**
 * Mutate the state to merge the delta
 */
const mergeDelta = (
  state: OpenAIChat.StreamResponseDelta,
  delta: OpenAIChat.StreamResponseDelta
) => {
  if (delta.role) {
    state.role = delta.role;
  }
  if (delta.content) {
    state.content = (state.content ?? "") + delta.content;
  }
  if (delta.function_call) {
    state.function_call = state.function_call || { name: "", arguments: "" };
    const { name, arguments: args } = delta.function_call || {};
    state.function_call!.name += name ?? "";
    state.function_call!.arguments += args ?? "";
  }
  return state;
};

export { mergeDelta };
