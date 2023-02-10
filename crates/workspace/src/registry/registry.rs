use url::Url;

/// Workspace registry is a remote Object storage like S3 where workspace
/// files are stored. The registry will store both the raw files as well
/// as bundled files.
///
/// Raw files are needed when editing a workspace. The compressed raw files
/// from the registry is loaded and uncompressed in the local filesystem
/// before being able to edit apps.
///
/// When a workspace apps are deployed, JS files are transpiled/bundled and
/// stored in the registry. When a request comes in, these bundles are fetched
/// into a local file system before before serving.
#[derive(Clone, Debug)]
pub struct Registry {
  /// Name of the workspace
  pub name: String,

  /// The directory to load the workspace in
  pub host: Url,
  // cache registry in local file system?
}

impl Registry {
  // TODO(sagar)
}
