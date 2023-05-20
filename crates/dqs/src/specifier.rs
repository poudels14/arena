use crate::types::widget::WidgetQuerySpecifier;
use anyhow::{anyhow, Result};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till};
use nom::combinator::eof;
use nom::error;
use nom::sequence::tuple;
use nom::IResult;
use tracing::instrument;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ParsedSpecifier {
  Env { app_id: String, widget_id: String },
  // WorkspaceMiddleware,
  // AppMiddleware,
  WidgetQuery(WidgetQuerySpecifier),
  // SavedQuery(String),
  Unknown,
}

fn take_until_slash(input: &str) -> IResult<&str, &str> {
  alt((take_till(|b| b == '/'), eof::<&str, error::Error<&str>>))(input)
}

impl ParsedSpecifier {
  #[instrument(name = "ParsedSpecifier::from", level = "trace")]
  pub fn from(specifier: &str) -> Result<Self> {
    Self::_parse(specifier)
      .map(|r| r.1)
      .map_err(|e| anyhow!("error parsing module specifier: {:?}", e))
  }

  fn _parse(input: &str) -> IResult<&str, Self> {
    let (input, _) = tag("workspace:///")(input)?;
    Self::parse_app_modules(input)
  }

  fn parse_app_modules(input: &str) -> IResult<&str, Self> {
    let (input, (_, app_id)) =
      tuple((tag("~/apps/"), take_until_slash))(input)?;

    let specifier = Self::parse_widget_query_source(app_id, input)?;
    Ok(specifier)
  }

  fn parse_widget_query_source<'a>(
    app_id: &'a str,
    input: &'a str,
  ) -> IResult<&'a str, Self> {
    let (input, (_, widget_id, _, field_name, maybe_env)) = tuple((
      tag("/widgets/"),
      take_until_slash,
      tag("/"),
      take_until_slash,
      alt((tag("/env"), eof)),
    ))(input)?;

    match maybe_env {
      "" => Ok((
        input,
        ParsedSpecifier::WidgetQuery(WidgetQuerySpecifier {
          app_id: app_id.to_string(),
          widget_id: widget_id.to_owned(),
          field_name: field_name.to_owned(),
        }),
      )),
      "/env" => Ok((
        input,
        ParsedSpecifier::Env {
          app_id: app_id.to_string(),
          widget_id: widget_id.to_string(),
        },
      )),
      _ => Ok((input, ParsedSpecifier::Unknown)),
    }
  }
}
