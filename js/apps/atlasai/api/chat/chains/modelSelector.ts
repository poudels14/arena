import { Workspace } from "@portal/workspace-sdk";
import { ChatOpenAI } from "@langchain/openai";
import { ChatGroq } from "@langchain/groq";
import { ChatOllama } from "@langchain/community/chat_models/ollama";
import { ChatAnthropic } from "@langchain/anthropic";

import { ChatPortalAI } from "./chat_model";

const getLLMModel = (
  model: Workspace.Model,
  options: { temperature: number }
) => {
  switch (model.provider) {
    case "groq": {
      return new ChatGroq({
        apiKey: model.config.http.apiKey,
        modelName: model.config.model.name || "mixtral-8x7b-32768",
        temperature: options.temperature,
      });
    }
    case "ollama": {
      return new ChatOllama({
        baseUrl:
          model.config.http.endpoint ||
          "http://localhost:1234/v1/chat/completions",
        model: model.config.model.name || "mistral",
        temperature: options.temperature,
      });
    }
    case "lmstudio": {
      return new ChatPortalAI({
        portalAIbaseUrl:
          model.config.http.endpoint || "http://localhost:1234/v1/",
        portalAIApiKey: "n/a",
        temperature: options.temperature,
      });
    }
    case "portal": {
      return new ChatPortalAI({
        portalAIbaseUrl: model.config.http.endpoint!,
        portalAIApiKey: "n/a",
        temperature: options.temperature,
        modelName: model.config.model.name,
      });
    }
    case "openai": {
      return new ChatOpenAI({
        modelName: model.config.model.name,
        temperature: options.temperature,
      });
    }
    case "anthropic": {
      return new ChatAnthropic({
        anthropicApiKey: model.config.http.apiKey,
        modelName: model.config.model.name,
        temperature: options.temperature,
      });
    }
    default:
      throw new Error("Unsupported model provider: " + model.provider);
  }
};

export { getLLMModel };
