const portal =
  // @ts-expect-error
  import.meta.env.MODE == "development"
    ? require("../../../../crates/target/debug/portal.node")
    : require("../../../../crates/target/release/portal.node");
portal.startPortalServer(42690);
