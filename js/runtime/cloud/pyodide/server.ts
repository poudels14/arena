import { serve } from "@arena/runtime/server";
// @ts-expect-error
import { pyodide } from "builtin://@arena/cloud/pyodide";
import { createRouter, procedure } from "@portal/server-core/router";
import z from "zod";

const py = await pyodide.loadPyodide({
  packages: ["numpy", "matplotlib", "portal"],
});

py.setStdout({ batched: (msg) => console.log(msg) });
py.setStderr({ batched: (msg) => console.log(msg) });
py.registerJsModule("portal_js", {
  matplotlib: {
    plot_filename: "portal_svg.svg",
  },
});

const p = procedure();
const router = createRouter({
  async middleware({ ctx, next }) {
    try {
      return await next({ ctx });
    } catch (e) {
      console.error(e);
      throw e;
    }
  },
  routes: {
    "/healthy": p.query(async ({ ctx }) => {
      return "Ok";
    }),
    "/exec/python": p
      .input(
        z.object({
          code: z.string(),
        })
      )
      .mutate(async ({ body, errors }) => {
        try {
          const data = await py.runPythonAsync(body.code);
          return { data, success: true };
        } catch (e: any) {
          return errors.internalServerError(e.toString());
        }
      }),
    // The POST body should be a valid code
    "/exec/python/raw": p.mutate(async ({ req, errors }) => {
      try {
        const data = await py.runPythonAsync(await req.text());
        return { data, success: true };
      } catch (e: any) {
        return errors.internalServerError(e.toString());
      }
    }),
  },
});

console.log("Python server started...");
serve({
  async fetch(req) {
    const res = await router.route(req, {});
    if (res) {
      return res;
    }
    return new Response("404 Not found", {
      status: 404,
    });
  },
});
