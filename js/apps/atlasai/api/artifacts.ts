import { merge, pick } from "lodash-es";
import mime from "mime";
import { p } from "./procedure";
import { convertToDataFrame } from "./utils/file";

const getArtifact = p.query(async ({ ctx, params, errors }) => {
  const artifact = await ctx.repo.artifacts.get({
    id: params.id,
  });

  if (!artifact) {
    return errors.notFound();
  }

  return merge(
    {
      content: artifact.file.content,
    },
    pick(artifact, "name", "size", "createdAt")
  );
});

const getArtifactContent = p.query(
  async ({ ctx, params, searchParams, errors, setHeader }) => {
    const artifact = await ctx.repo.artifacts.get({
      id: params.id,
    });

    if (!artifact) {
      return errors.notFound();
    }

    const content = Buffer.from(artifact.file.content, "base64");
    const contentType = mime.getType(artifact.name);
    if (searchParams.json == "true") {
      const contentString = content.toString("utf-8");
      let formattedData = "";
      if (contentType == "text/csv") {
        formattedData = convertToDataFrame(contentString);
      } else if (contentType == "text/plain") {
        formattedData = contentString;
      } else {
        return new Response(
          JSON.stringify({
            format: contentType,
            error: "Unsupported format",
          }),
          {
            status: 400,
            headers: {
              "Content-Type": "application/json",
            },
          }
        );
      }
      setHeader("Content-Type", "application/json");
      return {
        contentType,
        data: formattedData,
      };
    }
    if (contentType) {
      setHeader("Content-Type", contentType);
    }
    return content;
  }
);

export { getArtifact, getArtifactContent };
