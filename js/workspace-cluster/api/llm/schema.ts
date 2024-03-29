import { z } from "zod";

export const baseModelSchema = z.object({
  name: z.string().min(3),
  disabled: z.boolean().optional(),
  modalities: z.array(z.enum(["text", "image", "video"])),
  // family: z.enum([
  //   "openai:gpt-3",
  //   "openai:gpt-4",
  //   "mistral:mistral-7",
  //   "claude-instant-1",
  //   "anthropic:claude-2",
  //   "anthropic:claude-3-opus",
  //   "anthropic:claude-3-sonnet",
  //   "anthropic:claude-3-haiku",
  //   "unknown",
  // ]),
  type: z.enum([
    // llm is non chat model
    // "llm", // TODO
    "chat",
    // "image",
  ]),
});

export const ollamaModelSchema = baseModelSchema.merge(
  z.object({
    provider: z.literal("ollama"),
    config: z.object({
      http: z.object({
        endpoint: z.string().url(),
      }),
      model: z.object({
        name: z.string().min(1),
      }),
    }),
  })
);

export const lmStudioModelSchema = baseModelSchema.merge(
  z.object({
    provider: z.literal("lmstudio"),
    config: z.object({
      http: z.object({
        endpoint: z.string().url(),
      }),
    }),
  })
);

export const openAIModelSchema = baseModelSchema.merge(
  z.object({
    provider: z.literal("openai"),
    config: z.object({
      http: z.object({
        apiKey: z.string().min(1),
      }),
      model: z.object({
        name: z
          .enum([
            "gpt-4-0125-preview",
            "gpt-4-turbo-preview",
            "gpt-4-1106-preview",
            "gpt-4-vision-preview",
            "gpt-4-1106-vision-preview",
            "gpt-4",
            "gpt-4-0613",
            "gpt-4-32k",
            "gpt-4-32k-0613",
            "gpt-3.5-turbo-0125",
            "gpt-3.5-turbo",
            "gpt-3.5-turbo-1106",
          ])
          .or(z.string().min(4)),
      }),
    }),
  })
);

export const anthropicModelSchema = baseModelSchema.merge(
  z.object({
    provider: z.literal("anthropic"),
    config: z.object({
      http: z.object({
        apiKey: z.string().min(1),
      }),
      model: z.object({
        name: z
          .enum([
            "claude-3-opus-20240229",
            "claude-3-sonnet-20240229",
            "claude-3-haiku-20240307",
            "claude-2.1",
            "claude-2.0",
            "claude-instant-1.2",
          ])
          .or(z.string().min(4)),
      }),
    }),
  })
);

export const groqModelSchema = baseModelSchema.merge(
  z.object({
    provider: z.literal("groq"),
    config: z.object({
      http: z.object({
        apiKey: z.string().min(1),
      }),
      model: z.object({
        name: z.string().min(1),
      }),
    }),
  })
);
