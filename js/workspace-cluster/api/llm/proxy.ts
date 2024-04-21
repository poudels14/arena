import { z } from "zod";
import { protectedProcedure, p } from "../procedure";
import { env } from "../utils/env";

const llmProxy = protectedProcedure
  .input(
    z.object({
      provider: z.string(),
      model: z.string(),
      request: z.any(),
    })
  )
  .mutate(async ({ ctx, body, errors }) => {
    const response = await fetch("https://api.openai.com/v1/chat/completions", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer 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`,
      },
      body: JSON.stringify(body.request),
    });
    return response;
  });

const freeProxy = p.input(z.any({})).mutate(async ({ body }) => {
  return await fetch("https://api.groq.com/openai/v1/chat/completions", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${env.GROQ_API_KEY}`,
    },
    body: JSON.stringify({
      ...body,
      model: "llama3-70b-8192",
    }),
  });
});

export { llmProxy, freeProxy };
