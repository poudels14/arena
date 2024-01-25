import mime from "mime";
import { match } from "path-to-regexp";
import { p } from "./procedure";

const upload = p.mutate(async ({ ctx, searchParams, req, errors, form }) => {
  const template = await ctx.repo.appTemplates.fetchById(searchParams.appId);
  if (!template || template.ownerId != ctx.user?.id) {
    return errors.forbidden();
  }

  const files = await form.multipart(req);
  // TODO: check if existing version is already present
  for (const file of files) {
    const filepath = `/apps/${searchParams.appId}/${searchParams.version}/${file.filename}`;
    await ctx.s3Client.putObject("registry", filepath, {
      content: file.data,
    });
  }
  return { success: true };
});

const templateMatcher = match<any>("/apps/:appId/:appVersion/:type/(.*)");
const get = p.query(async ({ ctx, req, searchParams, errors }) => {
  const url = new URL(req.url);
  const pathname = url.pathname.substring("/registry".length);
  const pathMatch = templateMatcher(pathname);
  if (!pathMatch) {
    return errors.notFound();
  }

  if (
    pathMatch.params.type == "server" &&
    searchParams.API_KEY != ctx.env.REGISTRY_API_KEY
  ) {
    return errors.forbidden();
  }

  try {
    const object = await ctx.s3Client.getObject("registry", pathname);
    if (!object) {
      return errors.notFound();
    }

    return new Response(object.content, {
      headers: {
        "content-type": mime.getType(pathname)!,
      },
    });
  } catch (e) {
    console.error(e);
    return errors.notFound();
  }
});

export { upload, get };
