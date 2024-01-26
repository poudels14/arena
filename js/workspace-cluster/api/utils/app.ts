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
  const appId = slugify(app.id || uniqueId(14), {
    separator: "_",
    decamelize: false,
  });

  const newApp = await repo.apps.insert({
    id: appId,
    workspaceId: app.workspaceId,
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
    userId: user.id,
    workspaceId: app.workspaceId,
    appId,
    access: "owner",
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
