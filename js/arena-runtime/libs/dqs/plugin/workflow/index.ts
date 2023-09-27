import { createRouter, procedure, serve } from "@arena/runtime/server";
import { Manifest } from "@arena/sdk/plugins";
import { Status, createWorkflow } from "@arena/sdk/plugins/workflow";
// @ts-expect-error
import { manifest } from "@dqs/template/plugin";
import { createChatContext } from "./context";

const p = procedure<{}>();
const router = createRouter({
  async middleware({ ctx, next }) {
    try {
      return await next({ ctx });
    } catch (e) {
      console.error(e);
      return e;
    }
  },
  // Routes with `/_admin` prefix are only accessible by Arena cloud and
  // aren't exposed to the public
  routes: {
    "/_admin/healthy": p.query(async ({ ctx }) => {
      return "Ok";
    }),
    "/run": p.mutate(async ({ ctx, req, errors }) => {
      const { workflow, input } = await req.json();

      const { plugin } = manifest as Manifest;
      const { workflows } = plugin;
      const workflowConfig = workflows!.find((w) => (w.slug = workflow.slug));

      if (!workflowConfig) {
        return errors.notFound();
      }

      const steps = await workflowConfig.steps();
      const instance = createWorkflow().steps(steps).build();

      const result = await instance.start({
        state: {
          status: Status.IN_PROGRESS,
          steps: [],
        },
        input,
        ctx: createChatContext({
          creator: {
            app: {
              id: "",
            },
          },
        }),
        on(event) {
          console.log("EVENT =", event);
        },
      });

      return {
        result,
      };
    }),
  },
});

serve({
  async fetch(req) {
    const res = await router.route(req, {
      context: {},
    });

    if (res) {
      return res;
    }

    return new Response("404 Not found", {
      status: 404,
    });
  },
});
