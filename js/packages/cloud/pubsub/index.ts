const { ops } = Arena.core;

type Data =
  | {
      message: any;
    }
  | {
      state: any;
    }
  | {
      changeset: {
        /**
         * Reference id of the state that this changeset is based on
         */
        reference_id: string;

        /**
         * Sequence number of the changeset. It starts with 1
         */
        seqId: number;
        delta: any;
      };
    };

type IncomingEvent = {
  source:
    | {
        user: { id: string };
      }
    | { app: { id: string } };
  path: string;
  message: any;
};

const publish = async (data: Data, path?: string) => {
  return await ops.op_dqs_pubsub_publish(data, path);
};

async function subscribe(fn: (event: IncomingEvent) => void) {
  let event;
  while ((event = await ops.op_dqs_pubsub_subscribe())) {
    fn(event);
  }
}

export { publish, subscribe };
