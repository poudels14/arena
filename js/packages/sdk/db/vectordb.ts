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
  // a map of key, value to store arbiraty blobs corresponding to the document
  // this can be used to store original document, html, etc of the doc
  blobs?: Record<string, string | ArrayBuffer | null>;
};

export type DocumentEmbeddings = {
  start: number;
  end: number;
  vectors: number[];
  metadata?: Record<string, any>;
};

export type Document = {
  name: string;
  contentLength: number;
  chunksCount: number;
  metadata: any;
  content: any;
};

export type DocumentBlobs = Record<string, ArrayBuffer>;

export type SearchCollectionOptions = {
  includeChunkContent: boolean;
  contentEncoding?: string;
  minScore?: number;
  // number of bytes before the matched chunks to include in the response
  beforeContext?: number;
  // number of bytes after the matched chunks to include in the response
  afterContext?: number;
};

export type SearchCollectionResult = {
  score: number;
  documentId: string;
  chunkIndex: number;
  start: number;
  end: number;
  content: string;
  // empty if the matched embedding doesn't have metadata
  metadata: Record<string, any>;
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

  getDocumentBlobs(
    collectionId: string,
    documentId: string,
    keys: string[],
    encoding?: "base-64"
  ): Promise<DocumentBlobs>;

  setDocumentEmbeddings(
    collectionId: string,
    documentId: string,
    embeddings: DocumentEmbeddings[]
  ): Promise<void>;

  listDocuments(collectionId: string): Promise<Document[]>;

  deleteDocument(collectionId: string, documentId: string): Promise<void>;

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
    options?: SearchCollectionOptions
  ): Promise<SearchCollectionResult[]>;
};
