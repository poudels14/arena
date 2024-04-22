const portal =
  // @ts-expect-error
  import.meta.env.MODE == "development"
    ? require(`../../../../crates/target/debug/portal-${__APP_VERSION__}.node`)
    : require(`./portal-${__APP_VERSION__}.node`);

portal.startPortalServer(42690);
