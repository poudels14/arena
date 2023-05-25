use super::ResourceLoader;
use crate::config::PostgresSourceConfig;
use anyhow::{anyhow, bail, Result};
use handlebars::{no_escape, Handlebars};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::combinator::eof;
use nom::error::{self, ErrorKind};
use nom::sequence::tuple;
use nom::IResult;
use once_cell::sync::Lazy;
use serde_json::json;

static TEMPLATE: Lazy<Result<Handlebars>> = Lazy::new(|| {
  let mut reg = Handlebars::new();
  reg.set_strict_mode(true);
  reg.register_escape_fn(no_escape);
  reg.register_template_string(
    "SQL_QUERY_MODULE",
    include_str!("./postgres-query-template.js"),
  )?;

  Ok(reg)
});

impl ResourceLoader for PostgresSourceConfig {
  fn to_dqs_module(&self) -> Result<String> {
    let transpiled_query = self::transpile_query(&self.value)?;
    let prop_keys = self
      .metadata
      .args
      .iter()
      .rev()
      .fold("".to_owned(), |agg, v| format!("{} = null, {}", v, agg));

    TEMPLATE
      .as_ref()
      .expect("failed to load query template")
      .render(
        "SQL_QUERY_MODULE",
        &json!({
          "db": self.db,
          "query": transpiled_query,
          "paramKeys": format!("{{ {} }}", prop_keys)
        }),
      )
      .map_err(|e| anyhow!("{:?}", e))
  }
}

// this converts query from handle bar format to the format that can be used
// in slonik's sql tag
fn transpile_query(input: &str) -> Result<String> {
  let mut input = input;
  let mut transpiled = String::with_capacity(input.len() + 20);
  loop {
    match alt((take_until("{{"), eof::<&str, error::Error<&str>>))(input) {
      // if end of line is reached, add remaining input and return
      Err(nom::Err::Error(nom::error::Error { input, code })) => {
        if code == ErrorKind::Eof {
          transpiled.push_str(input);
          return Ok(transpiled);
        }
      }
      Ok((rem, pre)) => {
        transpiled.push_str(pre);
        match rem == "" {
          // if eof, return
          true => return Ok(transpiled),
          false => {
            input = rem;
            match self::parse_template_segment(input) {
              Ok((rem, (_open_brace, template, _close_brace))) => {
                transpiled.push_str("${");
                transpiled.push_str(template);
                transpiled.push_str("}");
                input = rem;
              }
              Err(e) => bail!("failed to parse sql query: {:?}", e),
            }
          }
        }
      }
      Err(e) => bail!("failed to parse sql query: {:?}", e),
    };
  }
}

fn parse_template_segment(input: &str) -> IResult<&str, (&str, &str, &str)> {
  tuple((tag("{{"), take_until("}}"), tag("}}")))(input)
}
