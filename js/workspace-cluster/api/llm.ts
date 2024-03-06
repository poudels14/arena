import { z } from "zod";
import { merge, pick } from "lodash-es";
import { protectedProcedure } from "./procedure";
import { uniqueId } from "@portal/sdk/utils/uniqueId";

const llmModelSchema = z.object({
  id: z.string(),
  name: z.string(),
  // true if it's a custom model
  custom: z.boolean(),
  disabled: z.boolean().optional(),
  requiresSubscription: z.boolean(),
  modalities: z.array(z.enum(["text", "image", "video"])),
  type: z.enum([
    // llm is non chat model
    "llm",
    "chat",
  ]),
  // TODO: until there's an universal interface, require the model family
  family: z.enum([
    "openai",
    "anthropic",
    "cohere",
    "gemini",
    "perplexity",
    "ollama",
    "togetherai",
    "mistral",
    "groq",
    "fireworks",
  ]),
  config: z.object({
    http: z.object({
      method: z.enum(["POST", "GET"]),
      endpoint: z.string().url(),
      headers: z.record(z.string()),
    }),
    // config specific to the model provider
    model: z.any(),
  }),
  quota: z.object({
    requests: z
      .object({
        // number of API request remaining
        remaining: z.number(),
      })
      .optional(),
  }),
});

type LLMModelSchema = z.infer<typeof llmModelSchema>;

const addCustomModel = protectedProcedure
  .input(
    z.object({
      workspaceId: z.string(),
      model: llmModelSchema.omit({
        id: true,
        custom: true,
        requiresSubscription: true,
        quota: true,
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
          "family",
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
          family: "openai",
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
          family: "openai",
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
