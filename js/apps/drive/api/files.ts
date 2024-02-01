import { merge, pick } from "lodash-es";
import { z } from "zod";
import mime from "mime";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import {
  ContentType,
  createDocument,
} from "@portal/internal-sdk/llm/documents";
import { p } from "./procedure";
import { createDocumentSplitter } from "@portal/sdk/llm/splitter";

const addDirectory = p
  .input(
    z.object({
      name: z.string(),
      parentId: z.string().nullable(),
    })
  )
  .mutate(async ({ ctx, body, errors }) => {
    if (body.parentId != null) {
      const parent = await ctx.repo.files.fetchById(body.parentId);
      if (!parent) {
        return errors.badRequest("Invalid parentId");
      }
    }

    const parentsChildren = await ctx.repo.files.listFiles({
      parentId: body.parentId,
    });
    if (parentsChildren.some((child) => child.name == body.name)) {
      return errors.badRequest("Duplicate directory name");
    }

    const directory = {
      id: uniqueId(),
      name: body.name,
      description: null,
      isDirectory: true,
      parentId: body.parentId,
      createdBy: ctx.user!.id,
      metadata: {},
      size: 0,
      file: null,
      createdAt: new Date(),
    };
    await ctx.repo.files.insert(directory);
    return merge(pick(directory, "id", "name", "parentId", "isDirectory"), {
      children: [],
    });
  });

const listDirectory = p.query(async ({ ctx, searchParams, errors }) => {
  const directoryId =
    searchParams.id == "null" ? null : searchParams.id || null;

  const directory = await ctx.repo.files.fetchById(directoryId);
  if (!directory && directoryId != null) {
    return errors.notFound("Directory not found");
  }
  const children = await ctx.repo.files.listFiles({
    parentId: directoryId,
  });
  return merge(pick(directory, "id", "name", "parentId", "isDirectory"), {
    children: children.map((child) => {
      return merge(pick(child, "id", "name", "parentId", "isDirectory"), {
        type: child.isDirectory ? null : mime.getType(child.name),
      });
    }),
  });
});

const uploadFiles = p.mutate(async ({ req, ctx, errors, form }) => {
  const formData = await form.multipart(req);
  const parentInput = formData.find((input) => input.name == "parentId");
  if (!parentInput) {
    return errors.badRequest("Missing `parentId` field");
  }

  const parentId = parentInput.data.toString();
  const parentDirectory = await ctx.repo.files.fetchById(
    parentId == "null" ? null : parentId
  );

  if (!parentDirectory && parentId != "null") {
    return errors.notFound("Directory not found");
  }

  const newFiles = [];
  const repo = await ctx.repo.transaction();
  try {
    for (const formInput of formData) {
      if (formInput.filename) {
        const fileContent = formInput.data.toString("base64");
        const file = {
          id: uniqueId(),
          name: formInput.filename,
          description: null,
          isDirectory: false,
          parentId: parentDirectory!.id,
          createdBy: ctx.user!.id,
          metadata: {},
          size: fileContent.length,
          file: {
            content: fileContent,
          },
          createdAt: new Date(),
        };

        const documentSplitter = createDocumentSplitter({
          async tokenize(content) {
            return await ctx.llm.embeddingsModel.tokenizeText(content, {
              truncate: false,
            });
          },
          maxTokenLength: 200,
          windowTerminationNodes: ["heading", "table", "code"],
        });

        const document = createDocument(
          mime.getType(formInput.filename) as ContentType,
          formInput
        );
        const documentChunks = await document.split(documentSplitter);
        const embeddings = await ctx.llm.embeddingsModel.generateEmbeddings(
          documentChunks.map((chunk) => chunk.content)
        );

        await repo.files.insert(file);
        newFiles.push(file);
        for (let index = 0; index < documentChunks.length; index++) {
          const chunk = documentChunks[index];
          await repo.embeddings.insert({
            id: uniqueId(),
            embeddings: embeddings[index],
            metadata: {
              start: chunk.position.start,
              end: chunk.position.end,
              chunk: chunk.metadata,
            },
            fileId: file.id,
            createdAt: file.createdAt,
          });
        }
      }
    }
    repo.commit();
  } finally {
    await repo.release();
  }
  return {
    files: newFiles.map((file) =>
      pick(file, "id", "name", "parentId", "isDirectory")
    ),
  };
});

export { addDirectory, listDirectory, uploadFiles };
