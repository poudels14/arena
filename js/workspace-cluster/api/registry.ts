import { p } from "./procedure";

const put = p.mutate(async ({ ctx, req, errors, form }) => {
  const x = await form.multipart(req);
  // TODO:
  //   z.object({
  //     appId: z.string(),
  //     version: z.string(),
  //     path: z.string(),
  //     content: z.any(),
  //   })
  console.log(x);
  // const object = await ctx.s3Client.putObject("registry", "", {
  //   content: Buffer.from([]),
  // });

  return errors.notFound();
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
        "content-type": object.headers["content-type"],
      },
    });
  } catch (e) {
    console.error(e);
    return errors.notFound();
  }
});

export { put, get };
