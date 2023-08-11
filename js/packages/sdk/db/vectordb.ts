import { AbstractDatabaseConfig } from "./common";

export type CollectionConfig = {
  dimension: number;
};

export type Collection = {
  id: string;
  documentsCount: number;
  dimension: number;
  metadata: any;
};

export type AddDocumentPayload = {
  content: string;
};

export type DocumentEmbeddings = {
  start: number;
  end: number;
  vectors: number[];
};

export type Document = {
  name: string;
  contentLength: number;
  chunksCount: number;
  metadata: any;
  content: any;
};

export type Config = AbstractDatabaseConfig<
  {
    /**
     * Database type
     */
    type: "arena-vectordb";
  },
  Client
>;

export type Client = {
  /**
   * Execute SQL query on the vector database
   */
  query(sql: string): Promise<any>;

  createCollection(
    collectionId: string,
    config: CollectionConfig
  ): Promise<void>;

  listCollections(): Promise<Collection[]>;

  getCollection(collectionId: string): Promise<Collection | undefined>;

  addDocument(
    collectionId: string,
    documentId: string,
    payload: AddDocumentPayload
  ): Promise<void>;

  getDocument(
    collectionId: string,
    documentId: string,
    encoding?: "utf-8"
  ): Promise<Document>;

  setDocumentEmbeddings(
    collectionId: string,
    documentId: string,
    embeddings: DocumentEmbeddings[]
  ): Promise<void>;

  listDocuments(collectionId: string): Promise<Document[]>;

  /**
   * Search embeddings in a collection by the given query vector
   *
   * @param collectionId collection id
   * @param queryVector the vector of the query
   * @param k The number of document chunks to return
   */
  searchCollection(
    collectionId: string,
    queryVector: number[],
    k: number,
    options?: {}
  ): Promise<
    {
      score: number;
      documentId: string;
      chunkIndex: number;
      start: number;
      end: number;
      content: string;
    }[]
  >;
};
