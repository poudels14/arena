const portal =
  // @ts-expect-error
  import.meta.env.MODE == "development"
    ? require(`../../../../crates/target/debug/portal-${_APP_VERSION}.node`)
    : require(`./portal-${_APP_VERSION}.node`);

portal.startPortalServer(42690);
