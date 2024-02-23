import { merge, pick } from "lodash-es";
import mime from "mime";
import { p } from "./procedure";
import { convertToDataFrame } from "./utils/file";

const listArtifacts = p.query(async ({ ctx }) => {
  const artifacts = await ctx.repo.artifacts.list({}, { limit: 20 });
  return artifacts.map((artifact) =>
    pick(artifact, "id", "name", "threadId", "metadata", "createdAt")
  );
});

const getArtifact = p.query(
  async ({ ctx, params, searchParams, errors, setHeader }) => {
    const artifact = await ctx.repo.artifacts.get({
      id: params.id,
    });

    if (!artifact) {
      return errors.notFound();
    }

    const content = Buffer.from(artifact.file.content, "base64");
    const contentType = mime.getType(artifact.name);
    let formattedContent = artifact.file.content;
    if (searchParams.json == "true") {
      const contentString = content.toString("utf-8");
      if (contentType == "text/csv") {
        formattedContent = convertToDataFrame(contentString);
      } else if (contentType == "text/plain") {
        formattedContent = contentString;
      }
    }

    setHeader("Cache-Control", "max-age=604800");
    return merge(
      {
        content: formattedContent,
        contentType,
      },
      pick(artifact, "name", "size", "createdAt")
    );
  }
);

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
    if (contentType) {
      setHeader("Content-Type", contentType);
    }
    setHeader("Cache-Control", "max-age=604800");
    return content;
  }
);

export { listArtifacts, getArtifact, getArtifactContent };
