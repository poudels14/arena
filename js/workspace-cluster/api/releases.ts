import { p } from "./procedure";

const hasDesktopAppUpdate = p.query(async ({ ctx, req, params, errors }) => {
  const { target, arch, currentVersion } = params;
  return new Response("No content", {
    status: 204,
  });
  // TODO
  // return {
  //   version: "0.3.0",
  //   pub_date: new Date().toISOString(),
  //   url: "https://mycompany.portal1235.com/myapp/releases/myrelease.tar.gz",
  //   signature: "Content of the relevant .sig file",
  //   notes: "These are some release notes",
  // };
});

export { hasDesktopAppUpdate };
