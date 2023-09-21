// @ts-expect-error
import { createHash } from "crypto";
import { pick } from "lodash-es";
import { renderToStringAsync } from "solid-js/web";
import { createComponent } from "solid-js";
import { createDocumentSplitter } from "@arena/llm/splitter";
import { HFTokenizer } from "@arena/cloud/llm";
import { p } from "../procedure";
import { DocumentEmbeddingsGenerator } from "../EmbeddingsGenerator";
import { uniqueId } from "../utils";
import Document from "./RenderDocument";
import { MarkdownDocument } from "../documents/markdown";
import { PdfDocument } from "../documents/pdf";

const FILE_TYPE = {
  MARKDOWN: "text/markdown",
  PDF: "application/pdf",
};

const EMBEDDINGS_MODEL = "thenlper/gte-small";
// Note: get-small support upto 512 tokens but leave some buffer :shrug:
const MAX_TOKEN_LENGTH = 400;

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
  const blobs = await vectordb.getDocumentBlobs("uploads", document.id, [
    "html",
  ]);

  return {
    ...pick(document, "id", "name", "contentType", "uploadedAt"),
    content: doc.content,
    html: blobs.html
      ? Buffer.from(blobs.html).toString("utf-8")
      : await renderToStringAsync(() =>
          createComponent(Document, {
            content: doc.content,
          })
        ),
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

const uploadDocuments = p.mutate(async ({ req, ctx, form, errors }) => {
  const { default: mainDb, vectordb } = ctx.dbs;

  const reqDocuments = await form.multipart(req);
  if (!reqDocuments.find((d) => Object.values(FILE_TYPE).includes(d.type))) {
    errors.badRequest(
      `Only [${Object.values(FILE_TYPE).join(",")}] type files are supported`
    );
  }

  const generator = new DocumentEmbeddingsGenerator();
  const splitter = createDocumentSplitter({
    async tokenize(content) {
      const tokenizer = await HFTokenizer.init({
        modelName: EMBEDDINGS_MODEL,
        truncate: false,
      });
      return await tokenizer.tokenize(content);
    },
    maxTokenLength: MAX_TOKEN_LENGTH,
    // TODO(sagar): use the chunk before and after since the following
    // termination nodes are used
    windowTerminationNodes: ["heading", "table", "code"],
  });

  const documents = reqDocuments.map((document) => {
    // NOTE(sagar): need to use ASCII encoding here since tokenizer uses
    // byte pair encoding and doesn't recognize utf-8. So, using utf-8
    // here causes the token position offset to not match with the
    // markdown nodes offsets
    const raw = document.data;
    const contentHash = createHash("sha256").update(raw).digest("hex");

    switch (document.type) {
      case FILE_TYPE.MARKDOWN:
        return {
          type: document.type,
          filename: document.filename,
          contentHash,
          document: new MarkdownDocument(document),
        };
      case FILE_TYPE.PDF:
        return {
          type: document.type,
          filename: document.filename,
          contentHash,
          document: new PdfDocument(document),
        };
      default:
        throw new Error("Unsupported document type");
    }
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
      .map(async ({ type, filename, contentHash, document }) => {
        const chunks = await document.split(splitter);
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
              filename,
              contentHash,
              type,
              filename,
              uploadedAt.getTime(),
            ]
          );

          await vectordb.addDocument("uploads", documentId, {
            content: await document.getContent(),
            blobs: {
              raw: await document.getRaw(),
              html: await document.getHtml(),
            },
          });

          await vectordb.setDocumentEmbeddings(
            "uploads",
            documentId,
            embeddings
          );
        });

        return {
          id: documentId,
          name: filename,
          filename: filename,
          contentType: type,
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
        ...pick(d, "id", "contentType", "uploadedAt"),
        name: d.name || d.filename,
      };
    }),
    new: newDocuments.map((d) => {
      return {
        ...pick(d, "id", "contentType", "uploadedAt"),
        name: d.name || d.filename,
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
