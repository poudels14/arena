import { createRouter, procedure, serve } from "@arena/runtime/server";
import { Manifest } from "@arena/sdk/plugins";
import { Status, createWorkflow } from "@arena/sdk/plugins/workflow";
// @ts-expect-error
import { manifest } from "@dqs/template/plugin";

const p = procedure<{}>();
const router = createRouter({
  // Routes with `/_admin` prefix are only accessible by Arena cloud and
  // aren't exposed to the public
  routes: {
    "/_admin/healthy": p.query(async ({ ctx }) => {
      return "Ok";
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

    const { workflowId, input } = await req.json();
    const { workflows } = manifest.plugin as Manifest["plugin"];
    const workflowConfig = workflows!.find((w) => (w.slug = workflowId));

    if (!workflowConfig) {
      return new Response("Not found", {
        status: 404,
      });
    }

    const instance = createWorkflow().steps(workflowConfig.steps).build();

    const result = await instance.start({
      state: {
        status: Status.IN_PROGRESS,
        steps: [],
      },
      input,
      ui: {
        async confirm(msg) {
          return true;
        },
        async form(html) {},
        async select(msg: string, options: any[]) {
          return options[0];
        },
      },
      on(event) {
        console.log("EVENT =", event);
      },
    });

    return new Response(
      JSON.stringify({
        result,
      })
    );
  },
});
