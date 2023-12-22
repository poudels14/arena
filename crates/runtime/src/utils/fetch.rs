// credit: deno
use deno_core::error::{type_error, AnyError};
use deno_fetch::CreateHttpClientOptions;
use deno_tls::SocketUse;
use http::header::USER_AGENT;
use http::HeaderMap;
use reqwest::redirect::Policy;
use reqwest::{Client, ClientBuilder};

/// This is used to create a custom client for deno `fetch`
/// This is copied from deno_fetch
pub fn get_default_http_client_builder(
  user_agent: &str,
  options: CreateHttpClientOptions,
) -> Result<ClientBuilder, AnyError> {
  let mut tls_config = deno_tls::create_client_config(
    options.root_cert_store,
    options.ca_certs,
    options.unsafely_ignore_certificate_errors,
    options.client_cert_chain_and_key,
    SocketUse::GeneralSsl,
  )?;

  let mut alpn_protocols = vec![];
  if options.http2 {
    alpn_protocols.push("h2".into());
  }
  if options.http1 {
    alpn_protocols.push("http/1.1".into());
  }
  tls_config.alpn_protocols = alpn_protocols;

  let mut headers = HeaderMap::new();
  headers.insert(USER_AGENT, user_agent.parse().unwrap());
  let mut builder = Client::builder()
    .redirect(Policy::none())
    .default_headers(headers)
    .use_preconfigured_tls(tls_config);

  if let Some(proxy) = options.proxy {
    let mut reqwest_proxy = reqwest::Proxy::all(&proxy.url)?;
    if let Some(basic_auth) = &proxy.basic_auth {
      reqwest_proxy =
        reqwest_proxy.basic_auth(&basic_auth.username, &basic_auth.password);
    }
    builder = builder.proxy(reqwest_proxy);
  }

  if let Some(pool_max_idle_per_host) = options.pool_max_idle_per_host {
    builder = builder.pool_max_idle_per_host(pool_max_idle_per_host);
  }

  if let Some(pool_idle_timeout) = options.pool_idle_timeout {
    builder = builder.pool_idle_timeout(
      pool_idle_timeout.map(std::time::Duration::from_millis),
    );
  }

  match (options.http1, options.http2) {
    (true, false) => builder = builder.http1_only(),
    (false, true) => builder = builder.http2_prior_knowledge(),
    (true, true) => {}
    (false, false) => {
      return Err(type_error("Either `http1` or `http2` needs to be true"))
    }
  }

  Ok(builder)
}
