import {
  AddDocumentPayload,
  Client,
  CollectionConfig,
  DocumentEmbeddings,
} from "@arena/sdk/db/vectordb";
import { assert } from "@arena/sdk/utils/assert";

const { opAsync } = Arena.core;
class VectorDatabase implements Client {
  #rid: number;
  #path: string;
  private constructor(rid: number, path: string) {
    this.#rid = rid;
    this.#path = path;
  }

  static async open(path: string) {
    const rid = await opAsync("op_cloud_vectordb_open", path);
    return new VectorDatabase(rid, path);
  }

  async createCollection(collectionId: string, config: CollectionConfig) {
    assert.notNil(collectionId, "Collection id");
    return await opAsync(
      "op_cloud_vectordb_create_collection",
      this.#rid,
      collectionId,
      config
    );
  }

  async listCollections() {
    return await opAsync("op_cloud_vectordb_list_collections", this.#rid);
  }

  async getCollection(collectionId: string): Promise<{
    id: string;
    documentsCount: number;
    dimension: number;
    metadata: any;
  }> {
    assert.notNil(collectionId, "Collection id");
    return await opAsync(
      "op_cloud_vectordb_get_collection",
      this.#rid,
      collectionId
    );
  }

  async addDocument(
    collectionId: string,
    documentId: string,
    payload: AddDocumentPayload
  ) {
    assert.notNil(collectionId, "Collection id");
    assert.notNil(documentId, "Document id");
    return await opAsync(
      "op_cloud_vectordb_add_document",
      this.#rid,
      collectionId,
      documentId,
      payload
    );
  }

  async getDocument(
    collectionId: string,
    documentId: string,
    encoding?: "utf-8"
  ) {
    assert.notNil(collectionId, "Collection id");
    assert.notNil(documentId, "Document id");
    return await opAsync(
      "op_cloud_vectordb_get_document",
      this.#rid,
      collectionId,
      documentId,
      encoding
    );
  }

  async setDocumentEmbeddings(
    collectionId: string,
    documentId: string,
    embeddings: DocumentEmbeddings[]
  ) {
    assert.notNil(collectionId, "Collection id");
    assert.notNil(documentId, "Document id");
    return await opAsync(
      "op_cloud_vectordb_set_document_embeddings",
      this.#rid,
      collectionId,
      documentId,
      embeddings
    );
  }

  async listDocuments(collectionId: string) {
    assert.notNil(collectionId, "Collection id");
    return await opAsync(
      "op_cloud_vectordb_list_documents",
      this.#rid,
      collectionId
    );
  }

  /**
   * Search embeddings in a collection by the given query vector
   *
   * @param collectionId collection id
   * @param queryVector the vector of the query
   * @param k The number of document chunks to return
   */
  async searchCollection(
    collectionId: string,
    queryVector: number[],
    k: number,
    options?: {}
  ) {
    assert.notNil(collectionId, "Collection id");
    return await opAsync(
      "op_cloud_vectordb_search_collection",
      this.#rid,
      collectionId,
      queryVector,
      k,
      options || {}
    );
  }
}

export { VectorDatabase };
