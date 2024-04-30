import path from "path";
import { Workspace } from "@portal/workspace-sdk";
import {
  AnthropicChat,
  GenericChatModel,
  Groq,
  OllamaChat,
} from "@portal/cortex/integrations/models";

const buildModelProvider = (model: Workspace.Model) => {
  switch (model.provider) {
    case "openai": {
      return new GenericChatModel({
        url: "https://api.openai.com/v1/chat/completions",
        model: model.config.model.name,
        apiKey: model.config.http.apiKey,
        // TODO
        contextLength: 8000,
      });
    }
    case "anthropic": {
      return new AnthropicChat({
        model: model.config.model.name,
        apiKey: model.config.http.apiKey!,
        version: "2023-06-01",
        // TODO
        contextLength: 8000,
      });
    }
    case "groq": {
      return new Groq({
        apiKey: model.config.http.apiKey,
        model: model.config.model.name || "mixtral-8x7b-32768",
      });
    }
    case "portal": {
      return new GenericChatModel({
        // TODO: remove /chat/completions suffix after atlasi ai is migrated to using
        // @portal/cortex and require that suffix in settings
        url: path.join(model.config.http.endpoint!, "chat/completions"),
        model: model.config.model.name,
        contextLength: 8000,
      });
    }
    case "ollama": {
      return new OllamaChat({
        // TODO: remove /chat/completions suffix after atlasi ai is migrated to using
        // @portal/cortex and require that suffix in settings
        url: path.join(
          model.config.http.endpoint || "http://localhost:11434/",
          "/api/chat"
        ),
        model: model.config.model.name,
        // TODO
        contextLength: 8000,
      });
    }
    case "lmstudio": {
      return new GenericChatModel({
        // TODO: remove /chat/completions suffix after atlasi ai is migrated to using
        // @portal/cortex and require that suffix in settings
        url: path.join(
          model.config.http.endpoint || "http://localhost:1234/v1/",
          "chat/completions"
        ),
        model: "unknown",
        // TODO
        contextLength: 8000,
      });
    }
    default:
      throw new Error("Unsupported model provider: " + model.provider);
  }
};

export { buildModelProvider };
