// @ts-expect-error
import { createHash } from "crypto";
import { pick } from "lodash-es";
import { Splitter } from "@arena/llm/splitter";
import { p } from "../procedure";
import { DocumentEmbeddingsGenerator } from "../EmbeddingsGenerator";
import { uniqueId } from "../utils";

const listDocuments = p.query(async ({ ctx }) => {
  const { default: sql } = ctx.dbs;
  const { rows: documents } = await sql.query<any>(`SELECT * FROM uploads`);
  return documents.map((doc) => {
    return {
      ...pick(doc, "id", "contentType"),
      name: doc.name || doc.filename,
      uploadedAt: new Date(doc.uploadedAt).toISOString(),
    };
  });
});

const getDocument = p.query(async ({ ctx, params, errors }) => {
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
});

const searchDocuments = p.query(async ({ ctx, searchParams }) => {
  const db = ctx.dbs.vectordb;

  const generator = new DocumentEmbeddingsGenerator();
  const embeddings = await generator.getTextEmbeddings([searchParams.query]);
  return await db.searchCollection("uploads", embeddings[0], 10, {
    includeChunkContent: true,
    contentEncoding: "utf-8",
    minScore: 0.7,
  });
});

const updateDocument = p.mutate(async ({ ctx, req, params, errors }) => {
  const { default: mainDb } = ctx.dbs;
  const { rows: documents } = await mainDb.query<any>(
    `SELECT * FROM uploads WHERE id = ?`,
    [params.documentId]
  );
  if (documents.length == 0) {
    return errors.notFound();
  }

  const body = await req.json();
  await mainDb.query(`UPDATE uploads SET name = ? WHERE id = ?`, [
    body.name,
    params.documentId,
  ]);
  return { success: true };
});

const deleteDocument = p.delete(async ({ ctx, params }) => {
  const { default: mainDb, vectordb } = ctx.dbs;
  await mainDb.transaction(async () => {
    await vectordb.deleteDocument("uploads", params.documentId);
    await mainDb.query<any>(`DELETE FROM uploads WHERE id = ?`, [
      params.documentId,
    ]);
  });
  return { success: true };
});

const uploadDocuments = p.mutate(async ({ req, ctx, form }) => {
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
      const contentHash = createHash("sha256").update(content).digest("hex");

      return { type, contentHash, content, document };
    });

  const { rows: existingDocs } = await mainDb.query(
    `SELECT * FROM uploads WHERE content_hash IN (${[...Array(documents.length)]
      .map((_) => "?")
      .join(",")})`,
    documents.map(({ contentHash }) => contentHash)
  );

  const newDocuments = await Promise.all(
    documents
      .filter(
        (d) => !existingDocs.find((ex: any) => ex.contentHash == d.contentHash)
      )
      .map(async ({ type, contentHash, content, document }) => {
        const chunks = await generator.split({
          type,
          content,
        } as Splitter.Document);

        const embeddings = await generator.getChunkEmbeddings(chunks);
        const documentId = uniqueId();
        const uploadedAt = new Date();
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
              uploadedAt.getTime(),
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
          id: documentId,
          name: document.filename,
          filename: document.filename,
          contentType: document.type,
          contentHash,
          chunks,
          embeddings,
          uploadedAt,
        };
      })
  );
  return {
    existing: existingDocs.map((d: any) => {
      return {
        ...pick(d, "id", "contentType"),
        name: d.name || d.filename,
        uploadedAt: new Date(d.uploadedAt).toISOString(),
      };
    }),
    new: newDocuments.map((d) => {
      return {
        ...pick(d, "id", "contentType"),
        name: d.name || d.filename,
        uploadedAt: d.uploadedAt.toISOString(),
      };
    }),
  };
});

export {
  listDocuments,
  getDocument,
  updateDocument,
  deleteDocument,
  searchDocuments,
  uploadDocuments,
};
