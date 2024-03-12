import { z } from "zod";
import { merge, pick } from "lodash-es";
import { protectedProcedure } from "./procedure";
import { uniqueId } from "@portal/sdk/utils/uniqueId";

const baseModelSchema = z.object({
  name: z.string().min(3),
  disabled: z.boolean().optional(),
  modalities: z.array(z.enum(["text", "image", "video"])),
  type: z.enum([
    // llm is non chat model
    // "llm", // TODO
    "chat",
    // "image",
  ]),
});

const ollamaModelSchema = baseModelSchema.merge(
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

const lmStudioModelSchema = baseModelSchema.merge(
  z.object({
    provider: z.literal("lmstudio"),
    config: z.object({
      http: z.object({
        endpoint: z.string().url(),
      }),
    }),
  })
);

const openAIModelSchema = baseModelSchema.merge(
  z.object({
    provider: z.literal("openai"),
    config: z.object({
      http: z.object({
        apiKey: z.string().min(1),
      }),
      model: z.object({
        name: z.enum([
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
        ]),
      }),
    }),
  })
);

const anthropicModelSchema = baseModelSchema.merge(
  z.object({
    provider: z.literal("anthropic"),
    config: z.object({
      http: z.object({
        apiKey: z.string().min(1),
      }),
      model: z.object({
        name: z.enum([
          "claude-3-opus-20240229",
          "claude-3-sonnet-20240229",
          "claude-2.1",
          "claude-2.0",
          "claude-instant-1.2",
        ]),
      }),
    }),
  })
);

const groqModelSchema = baseModelSchema.merge(
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

const customLLMModelSchema = z.union([
  ollamaModelSchema,
  lmStudioModelSchema,
  openAIModelSchema,
  anthropicModelSchema,
  groqModelSchema,
]);

const llmModelSchema = z
  .object({
    id: z.string(),
    // true if it's a custom model
    custom: z.boolean(),
    requiresSubscription: z.boolean(),
    quota: z.object({
      requests: z
        .object({
          // number of API request remaining
          remaining: z.number(),
        })
        .optional(),
    }),
  })
  .and(customLLMModelSchema);

type LLMModelSchema = z.infer<typeof llmModelSchema>;

const addCustomModel = protectedProcedure
  .input(
    z.object({
      workspaceId: z.string(),
      model: customLLMModelSchema,
    })
  )
  .mutate(async ({ ctx, body, errors }) => {
    const isWorkspaceMember = await ctx.repo.workspaces.isWorkspaceMember({
      userId: ctx.user.id,
      workspaaceId: body.workspaceId,
    });
    if (!isWorkspaceMember) {
      return errors.forbidden();
    }

    const newModel = await ctx.repo.settings.insert({
      id: uniqueId(21),
      userId: ctx.user.id,
      workspaceId: body.workspaceId,
      metadata: {
        ...body.model,
        custom: true,
      },
      namespace: "llm/models",
    });
    return newModel;
  });

const listModels = protectedProcedure.query(
  async ({ ctx, searchParams, errors }) => {
    const user = await ctx.repo.users.fetchById(ctx.user.id);
    if (!user) {
      return errors.forbidden();
    }

    const settings = await ctx.repo.settings.list({
      userId: user.id,
      workspaceId: searchParams.workspaceId,
      namespace: "llm/models",
    });

    const models = settings.map((s) => {
      return merge(
        {
          id: s.id,
        },
        pick(
          s.metadata,
          "name",
          "custom",
          "disabled",
          "modalities",
          "type",
          "provider",
          "config"
        )
      ) as LLMModelSchema;
    });
    const customModels = models.filter((model) => model.custom);

    const gpt35UserConfig = models.find(
      (m) => !m.custom && m.id == "openai-gpt-3.5"
    );
    const gpt4UserConfig = models.find(
      (m) => !m.custom && m.id == "openai-gpt-4"
    );
    const availableModels: LLMModelSchema[] = [
      merge(
        {
          id: "openai-gpt-3.5",
          name: "OpenAI GPT 3.5",
          custom: false,
          disabled: false,
          modalities: ["text"],
          type: "chat",
          provider: "openai",
          quota: {},
          config: {
            http: {
              method: "POST",
              endpoint: "",
              headers: {},
            },
          },
        },
        gpt35UserConfig
      ),
      merge(
        {
          id: "openai-gpt-4",
          name: "OpenAI GPT 4",
          custom: false,
          disabled: false,
          requiresSubscription: true,
          modalities: ["text"],
          type: "chat",
          provider: "openai",
          config: {
            http: {
              method: "POST",
              endpoint: "",
              headers: {},
            },
          },
          quota: {
            requests: {
              remaining: 0,
            },
          },
        },
        gpt4UserConfig
      ),
      ...customModels,
    ];

    return availableModels;
  }
);

const updateModel = protectedProcedure
  .input(
    z.object({
      workspaceId: z.string(),
      id: z.string(),
      metadata: z.object({
        disabled: z.boolean().optional(),
      }),
    })
  )
  .mutate(async ({ ctx, body, errors }) => {
    const isWorkspaceMember = await ctx.repo.workspaces.isWorkspaceMember({
      userId: ctx.user.id,
      workspaaceId: body.workspaceId,
    });
    if (!isWorkspaceMember) {
      return errors.forbidden();
    }

    const settings = await ctx.repo.settings.getById(body.id);
    if (!settings || settings.workspaceId != body.workspaceId) {
      return errors.notFound();
    }
    await ctx.repo.settings.updateById(
      body.id,
      merge(settings.metadata, body.metadata)
    );
    return { success: true };
  });

const deleteModel = protectedProcedure
  .input(
    z.object({
      workspaceId: z.string(),
      id: z.string(),
    })
  )
  .mutate(async ({ ctx, body, errors }) => {
    const isWorkspaceMember = await ctx.repo.workspaces.isWorkspaceMember({
      userId: ctx.user.id,
      workspaaceId: body.workspaceId,
    });
    if (!isWorkspaceMember) {
      return errors.forbidden();
    }

    const settings = await ctx.repo.settings.getById(body.id);
    if (!settings || settings.workspaceId != body.workspaceId) {
      return errors.notFound();
    }
    await ctx.repo.settings.archiveById(body.id);
    return { success: true };
  });

export { addCustomModel, listModels, updateModel, deleteModel };
