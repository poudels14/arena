import { DbTaskExecution } from "~/api/repo/tasks";

type Manifest<Args> = {
  name: string;
  description: string;
  // zod schema
  schema: any;
  config?: {
    setup?: {
      disableFollowUp?: boolean;
    };
  };
  start(options: {
    args: Args;
    env: Record<string, string>;
    utils: {
      uploadArtifact(artifact: Artifact): Promise<{ id: string; name: string }>;
    };
    prompt: (message: string) => Promise<void>;
    setStatus: (status: DbTaskExecution["status"]) => Promise<void>;
    setState: (state: any, status?: DbTaskExecution["status"]) => Promise<void>;
  }): Promise<void>;
};

type Artifact = {
  id: string;
  path: string;
  size: number;
  content: string;
};

export type { Manifest, Artifact };
