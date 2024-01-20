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
  GetObjectResponse,
};
