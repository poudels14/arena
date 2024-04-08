import { z } from "zod";
import { pick } from "lodash-es";
import { p } from "../procedure";
import { uniqueId } from "@portal/sdk/utils/uniqueId";

const listProfiles = p.query(async ({ ctx }) => {
  const profiles = await ctx.repo.promptProfiles.list({});
  return profiles.map((profile) => {
    return pick(
      profile,
      "id",
      "name",
      "description",
      "bookmarked",
      "default",
      "createdAt"
    );
  });
});

const getProfile = p.query(async ({ ctx, params, errors }) => {
  const profile = await ctx.repo.promptProfiles.get(params.id);
  if (!profile) {
    return errors.notFound();
  }
  return pick(
    profile,
    "id",
    "name",
    "description",
    "template",
    "bookmarked",
    "default",
    "createdAt"
  );
});

const addProfile = p
  .input(
    z.object({
      name: z.string().min(3),
      description: z.string().optional(),
      prompt: z.string().min(5),
    })
  )
  .mutate(async ({ ctx, body }) => {
    const now = new Date();
    const profile = {
      id: uniqueId(22),
      name: body.name,
      description: body.description || "",
      template: body.prompt,
      bookmarked: false,
      default: false,
      metadata: {},
      createdAt: now,
      lastUsedAt: now,
    };
    await ctx.repo.promptProfiles.insert(profile);
    return profile;
  });

const updateProfile = p
  .input(
    z.object({
      name: z.string().min(3).optional(),
      description: z.string().optional(),
      prompt: z.string().min(5).optional(),
      default: z.boolean().optional(),
    })
  )
  .mutate(async ({ ctx, params, body, errors }) => {
    const existingProfile = await ctx.repo.promptProfiles.get(params.id);
    if (!existingProfile) {
      return errors.notFound();
    }
    if (body.default) {
      await ctx.repo.promptProfiles.clearDefault();
    }

    const { prompt: template, ...updates } = body;
    await ctx.repo.promptProfiles.update({
      id: params.id,
      template,
      ...updates,
    });
    return {
      ...existingProfile,
      ...body,
    };
  });

const deleteProfile = p.mutate(async ({ ctx, params, errors }) => {
  const existingProfile = await ctx.repo.promptProfiles.get(params.id);
  if (!existingProfile) {
    return errors.notFound();
  }
  await ctx.repo.promptProfiles.deleteById(existingProfile.id);
  return {
    success: true,
  };
});

export { addProfile, getProfile, listProfiles, updateProfile, deleteProfile };
