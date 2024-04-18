import { createRouter, procedure } from "@portal/server-core/router";
import { DatabaseClients } from "@portal/deploy/db";

const p = procedure<{ dbs: DatabaseClients<{}> }>();
const router = createRouter({
  // Routes with `/_admin` prefix are only accessible by Arena cloud and
  // aren't exposed to the public
  routes: {
    "/_admin/healthy": p.query(async ({ ctx }) => {
      return "Ok";
    }),
    // TODO(sagar): add bunch of endpoints under `/_metadata` that provides
    // information about the app like, what permissions this app needs
    // when installing, what resources are needed to be installed in the
    // workspace that this app will need to access, etc
    "/_admin/metadata/permissions": p.query(async () => {}),
    // TODO(sagar): this will return the schema of the routes this app has
    // which can be used by LLM models.
    // TODO: maybe this endpoint should be under `/_admin`?
    "/_admin/metadata/api/schemas": p.query(async () => {}),
  },
});

export { router };
