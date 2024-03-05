import { BaseChatMessageHistory } from "@langchain/core/chat_history";
import {
  AIMessage,
  BaseMessage,
  HumanMessage,
  SystemMessage,
} from "@langchain/core/messages";
import { ChatMessage } from "~/api/repo/chatMessages";

class AtalasChatMessageHistory extends BaseChatMessageHistory {
  messages: ChatMessage[];
  constructor(messages: ChatMessage[]) {
    super();
    this.messages = messages;
  }

  async addMessage(message: BaseMessage): Promise<void> {}

  async addAIMessage(message: string): Promise<void> {
    throw new Error("addAIMessage not implemented");
  }

  async addUserMessage(message: string): Promise<void> {
    throw new Error("addUserMessage not implemented");
  }

  async addAIChatMessage(message: string): Promise<void> {
    throw new Error("addAIChatMessage not implemented");
  }

  async getMessages(): Promise<BaseMessage[]> {
    return this.messages
      .filter((m) => m.message.content)
      .map((message) => {
        if (message.role == "system") {
          return new SystemMessage(message.message.content!);
        } else if (message.role == "ai") {
          return new AIMessage(message.message.content!);
        } else {
          return new HumanMessage(message.message.content!);
        }
      });
  }

  async clear(): Promise<void> {
    throw new Error("clear not implemented");
  }

  get lc_namespace() {
    return ["atlasai"];
  }
}

export { AtalasChatMessageHistory };
