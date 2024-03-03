import slugify from "@sindresorhus/slugify";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { Repo } from "../repo";
import { addDatabase } from "./database";

const addApp = async (
  repo: Repo,
  user: { id: string },
  app: {
    id?: string;
    workspaceId: string;
    name: string;
    description?: string;
    template: {
      id: string;
      version: string;
    };
  }
) => {
  // start with `app` to make sure db name doesn't start with number
  const appId =
    "app_" +
    slugify(app.id || uniqueId(14), {
      separator: "_",
      decamelize: false,
    });

  const newApp = await repo.apps.insert({
    id: appId,
    workspaceId: app.workspaceId,
    ownerId: user.id,
    name: app.name,
    slug: slugify(app.name, {
      separator: "_",
    }),
    description: app.description || "",
    template: app.template,
    createdBy: user.id,
    config: {},
  });

  await repo.acl.addAccess({
    id: uniqueId(19),
    userId: user.id,
    workspaceId: app.workspaceId,
    appId,
    appTemplateId: app.template.id,
    accessGroup: "owner",
    metadata: {
      filters: [
        {
          command: "*",
          // access all tables
          table: "*",
          // access all tables
          condition: "*",
        },
      ],
    },
    resourceId: "",
  });

  const database = await addDatabase(repo, {
    id: appId,
    workspaceId: app.workspaceId,
    appId,
    user: "app",
  });

  return {
    app: newApp,
    database,
  };
};

export { addApp };
