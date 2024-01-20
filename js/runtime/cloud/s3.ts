declare var Arena;

type ClientOptions = {
  region: {
    Custom: {
      region: string;
      endpoint: string;
    };
  };
  credentials: {
    access_key: string;
    secret_key: string;
  };
  withPathStyle: boolean;
};

type ListBucketOptions = {
  prefix?: string;
  delimiter?: string;
};

type ListBucketResponse = {
  objects: {
    lastModified: string;
    eTag: string;
    storageClass: string;
    key: string;
    size: number;
  };
};

type PubObjectRequest = {
  path: string;
  // @ts-expect-error
  content: Buffer;
};

type HeadObjectResponse = {
  accept_ranges: string | undefined;
  content_disposition: string | undefined;
  content_encoding: string | undefined;
  content_length: number | undefined;
  content_type: string | undefined;
  e_tag: string | undefined;
  expiration: string | undefined;
  expires: string | undefined;
  last_modified: string | undefined;
  metadata: Record<string, string>;
  parts_count: number | undefined;
  restore: string | undefined;
  storage_class: string | undefined;
  version_id: string | undefined;
};

type GetObjectResponse = {
  headers: Record<string, string>;
  content: Uint8Array;
};

const { core } = Arena;
class Client {
  #id: number;
  constructor(options: ClientOptions) {
    this.#id = core.ops.op_cloud_s3_create_client(options);
  }

  async createBucket(name: string) {
    return await core.opAsync("op_cloud_s3_create_bucket", this.#id, { name });
  }

  async listBucket(
    name: string,
    options?: ListBucketOptions
  ): Promise<ListBucketResponse> {
    return await core.opAsync(
      "op_cloud_s3_list_bucket",
      this.#id,
      name,
      options || {}
    );
  }

  async putObject(bucket: string, request: PubObjectRequest) {
    return await core.opAsync("op_cloud_s3_put_object", this.#id, bucket, {
      ...request,
      // @ts-expect-error
      content: Buffer.from(request.content),
    });
  }

  async headObject(bucket: string, path: string): Promise<HeadObjectResponse> {
    return await core.opAsync(
      "op_cloud_s3_head_object",
      this.#id,
      bucket,
      path
    );
  }

  async getObject(bucket: string, path: string): Promise<GetObjectResponse> {
    return await core.opAsync("op_cloud_s3_get_object", this.#id, bucket, path);
  }
}

export { Client };
export type {
  ClientOptions,
  ListBucketOptions,
  ListBucketResponse,
  PubObjectRequest,
  HeadObjectResponse,
  GetObjectResponse,
};
