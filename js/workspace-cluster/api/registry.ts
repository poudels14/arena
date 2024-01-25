import mime from "mime";
import { p } from "./procedure";

const upload = p.mutate(async ({ ctx, searchParams, req, errors, form }) => {
  const files = await form.multipart(req);
  // TODO: auth
  // TODO: check if existing version is already present
  for (const file of files) {
    const filepath = `/apps/${searchParams.appId}/${searchParams.version}/${file.filename}`;
    await ctx.s3Client.putObject("registry", filepath, {
      content: file.data,
    });
  }
  return { success: true };
});

const get = p.query(async ({ ctx, req, errors }) => {
  const url = new URL(req.url);
  const pathname = url.pathname.substring("/registry/".length);

  // TODO: check for access token before allowing access to server code
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
