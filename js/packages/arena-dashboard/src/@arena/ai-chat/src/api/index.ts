// @ts-expect-error
import { createHash } from "crypto";
import { createRouter, procedure } from "@arena/runtime/server";
import { pick } from "lodash-es";
import { Splitter } from "@arena/llm/splitter";
import { DatabaseClients } from "@arena/sdk/db";
import { uniqueId as generateUniqueId } from "@arena/sdk/utils/uniqueId";
import { databases } from "../../server";
import { DocumentEmbeddingsGenerator } from "./EmbeddingsGenerator";
import { chatCompletion } from "./OpenAI";
import { generateSystemPrompt } from "./prompt";

const uniqueId = () => generateUniqueId(25);
const p = procedure<{
  user: any;
  dbs: DatabaseClients<typeof databases>;
}>().use(async ({ ctx, next }) => {
  return await next({ ctx });
  // TODO(sagar): do auth
});
const router = createRouter({
  prefix: "/api",
  async middleware({ ctx, next }) {
    try {
      return await next({ ctx });
    } catch (e) {
      console.error(e);
      throw e;
    }
  },
  routes: {
    "/chat/:sessionId/send": p.mutate(async ({ ctx, params, req, errors }) => {
      let request: {
        id: string;
        message: string;
      };
      try {
        request = await req.json();
      } catch (e) {
        return "Error parsing request body";
      }

      const sessionId = params.sessionId;
      if (!request.message) {
        errors.badRequest("Message can't be empty");
      }

      request.id = request.id || uniqueId();

      const { rows: sessions } = await ctx.dbs.default.query(
        `SELECT * FROM chat_sessions WHERE id = ?`,
        [sessionId]
      );
      if (sessions.length == 0) {
        await ctx.dbs.default.query(
          `INSERT INTO chat_sessions(id) VALUES (?)`,
          [sessionId]
        );
      }

      await ctx.dbs.default.query(
        `INSERT INTO chat_messages(id, session_id, role, message, timestamp) VALUES (?,?,?,?,?)`,
        [
          request.id,
          sessionId,
          ctx.user?.id || "user",
          request.message,
          new Date().getTime(),
        ]
      );

      // TODO(sagar)
      const generator = new DocumentEmbeddingsGenerator();
      const embeddings = await generator.getTextEmbeddings([request.message]);
      const vectorSearchResult = await ctx.dbs.vectordb.searchCollection(
        "uploads",
        embeddings[0],
        4,
        {
          includeChunkContent: true,
          contentEncoding: "utf-8",
        }
      );

      const aiResponseTime = new Date();
      const aiResponseId = uniqueId();

      const openAiUserId = encodeToBase64(
        Buffer.from(JSON.stringify({ queryId: request.id }))
      );

      const [llmQueryRequest, aiResponseStream] = await chatCompletion({
        userId: openAiUserId,
        message: {
          system: {
            content: generateSystemPrompt({
              documents: vectorSearchResult,
            }),
          },
          query: request.message,
        },
      });

      // async function* a() {
      //   const { rows } = await ctx.dbs.default.query<any>(
      //     `SELECT * FROM chat_messages`
      //   );

      //   const content = rows[0].message;
      //   for (let i = 0; i < content.length; i += 4) {
      //     await new Promise((r) => {
      //       setTimeout(() => r(null), 100);
      //     });

      //     yield {
      //       json: {
      //         choices: [
      //           {
      //             delta: {
      //               content: content.substring(i, i + 4),
      //             },
      //           },
      //         ],
      //       },
      //     };
      //   }
      // }

      // const aiResponseStream = a();

      let aiResponse = "";
      const stream = new ReadableStream({
        async start(controller) {
          controller.enqueue(
            JSON.stringify({
              id: aiResponseId,
              timestamp: aiResponseTime.getTime(),
            })
          );
          try {
            for await (const data of aiResponseStream) {
              if (data.json) {
                const { content } = data.json.choices[0].delta;
                if (content) {
                  controller.enqueue(
                    JSON.stringify({
                      text: content,
                    })
                  );
                  aiResponse += content;
                }
              }
            }
            await ctx.dbs.default.query(
              `INSERT INTO chat_messages
              (id, session_id, parent_id, role, message, model, metadata, timestamp)
            VALUES (?,?,?,?,?,?)`,
              [
                aiResponseId,
                sessionId,
                request.id,
                "ai",
                aiResponse,
                llmQueryRequest.model,
                JSON.stringify({
                  documents: vectorSearchResult.map((r) =>
                    pick(r, "score", "documentId", "chunkId")
                  ),
                }),
                aiResponseTime.getTime(),
              ]
            );
          } catch (e) {
            controller.error(e);
          }
          controller.close();
        },
      });

      return new Response(stream, {
        status: 200,
        headers: [["content-type", "text/event-stream"]],
      });
    }),
    "/chat/sessions": p.query(async ({ ctx }) => {
      const { rows } = await ctx.dbs.default.query(
        `SELECT * FROM chat_sessions`
      );
      return rows;
    }),
    "/chat/:sessionId/messages": p.query(async ({ ctx, params }) => {
      const { rows } = await ctx.dbs.default.query(
        `SELECT * FROM chat_messages where session_id = ? ORDER BY timestamp`,
        [params.sessionId]
      );
      return rows;
    }),
    "/chat/:sessionId/messages/:id": p.delete(async ({ ctx, params }) => {
      await ctx.dbs.default.query(
        `DELETE FROM chat_messages where id = ? AND session_id = ?`,
        [params.id, params.sessionId]
      );
      return { success: true };
    }),
    "/documents": p.query(async ({ ctx }) => {
      const { default: sql } = ctx.dbs;
      const { rows: documents } = await sql.query<any>(`SELECT * FROM uploads`);
      return documents.map((doc) => {
        return {
          ...pick(doc, "id", "contentType"),
          name: doc.name || doc.filename,
          uploadedAt: new Date(doc.uploadedAt).toISOString(),
        };
      });
    }),
    "/documents/:documentId": p.query(async ({ ctx, params, errors }) => {
      const { default: sql, vectordb } = ctx.dbs;
      const { rows: documents } = await sql.query<any>(
        `SELECT * FROM uploads WHERE id = ?`,
        [params.documentId]
      );
      if (documents.length == 0) {
        return errors.notFound();
      }

      const document = documents[0];
      const doc = await vectordb.getDocument("uploads", document.id, "utf-8");
      return {
        ...document,
        uploadedAt: new Date(document.uploadedAt).toISOString(),
        content: doc.content,
      };
    }),
    "/documents/search": p.query(async ({ ctx, searchParams }) => {
      const db = ctx.dbs.vectordb;

      const generator = new DocumentEmbeddingsGenerator();
      const embeddings = await generator.getTextEmbeddings([
        searchParams.query,
      ]);
      return await db.searchCollection("uploads", embeddings[0], 10, {
        includeChunkContent: true,
        contentEncoding: "utf-8",
      });
    }),
    "/documents/upload": p.mutate(async ({ req, ctx, form }) => {
      const { default: mainDb, vectordb } = ctx.dbs;

      const reqDocuments = await form.multipart(req);
      const generator = new DocumentEmbeddingsGenerator();
      const documents = reqDocuments
        .map((document) => ({
          // TODO(sagar): improve this using "mime" package
          type: document.type == "text/markdown" ? "markdown" : null,
          document,
        }))
        .filter((d) => d.type == "markdown")
        .map(({ type, document }) => {
          // NOTE(sagar): need to use ASCII encoding here since tokenizer uses
          // byte pair encoding and doesn't recognize utf-8. So, using utf-8
          // here causes the token position offset to not match with the
          // markdown nodes offsets
          const content = document.data.toString("ascii");
          const contentHash = createHash("sha256")
            .update(content)
            .digest("hex");

          return { type, contentHash, content, document };
        });

      const { rows: existingDocs } = await mainDb.query(
        `SELECT * FROM uploads WHERE content_hash IN (${[
          ...Array(documents.length),
        ]
          .map((_) => "?")
          .join(",")})`,
        documents.map(({ contentHash }) => contentHash)
      );

      const newDocuments = await Promise.all(
        documents
          .filter(
            (d) =>
              !existingDocs.find((ex: any) => ex.contentHash == d.contentHash)
          )
          .map(async ({ type, contentHash, content, document }) => {
            const chunks = await generator.split({
              type,
              content,
            } as Splitter.Document);

            const embeddings = await generator.getChunkEmbeddings(chunks);
            const documentId = uniqueId();
            await mainDb.transaction(async () => {
              await mainDb.query(
                `INSERT INTO uploads (
                      id, name, content_hash,
                      content_type, filename, uploaded_at
                    )
                    VALUES(?, ?, ?, ?, ?, ?)`,
                [
                  documentId,
                  document.filename,
                  contentHash,
                  type,
                  document.filename,
                  new Date().getTime(),
                ]
              );

              await vectordb.addDocument("uploads", documentId, {
                content,
              });

              await vectordb.setDocumentEmbeddings(
                "uploads",
                documentId,
                embeddings
              );
            });

            return {
              name: document.name,
              filename: document.filename,
              contentHash,
              chunks,
              embeddings,
            };
          })
      );
      return {
        existing: existingDocs.map((d) => pick(d, "filename", "contentHash")),
        new: newDocuments.map((d) => pick(d, "filename", "contentHash")),
      };
    }),
  },
});

export { router };
