import { Workspace } from "@portal/workspace-sdk";
import { ChatOpenAI } from "@langchain/openai";
import { ChatGroq } from "@langchain/groq";
import { ChatOllama } from "@langchain/community/chat_models/ollama";
import { ChatAnthropic } from "@langchain/anthropic";

import { ChatPortalAI } from "./chat_model";

const getLLMModel = (model: Workspace.Model) => {
  switch (model.provider) {
    case "groq": {
      return new ChatGroq({
        apiKey: model.config.http.apiKey,
        modelName: model.config.model.name || "mixtral-8x7b-32768",
      });
    }
    case "ollama": {
      return new ChatOllama({
        baseUrl:
          model.config.http.endpoint ||
          "http://localhost:1234/v1/chat/completions",
        model: model.config.model.name || "mistral",
      });
    }
    case "lmstudio": {
      return new ChatPortalAI({
        portalAIbaseUrl:
          model.config.http.endpoint || "http://localhost:1234/v1/",
        portalAIApiKey: "n/a",
      });
    }
    case "openai": {
      return new ChatOpenAI({
        modelName: model.config.model.name,
      });
    }
    case "anthropic": {
      return new ChatAnthropic({
        anthropicApiKey: model.config.http.apiKey,
        modelName: model.config.model.name,
      });
    }
    default:
      throw new Error("Unsupported model provider: " + model.provider);
  }
};

export { getLLMModel };
