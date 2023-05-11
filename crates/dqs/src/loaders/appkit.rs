use anyhow::{anyhow, Result};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till};
use nom::combinator::eof;
use nom::IResult;

#[derive(Debug)]
pub struct WidgetQuerySource {
  pub widget_id: String,
  pub field_name: String,
}

#[allow(dead_code)]
pub enum ModuleSource {
  Env,
  WorkspaceMiddleware,
  AppMiddleware,
  WidgetQuery(WidgetQuerySource),
  SavedQuery(String),
  Unknown,
}

impl ModuleSource {
  pub fn parse(specifier: &str) -> Result<Self> {
    Self::_parse(specifier)
      .map(|r| r.1)
      .map_err(|e| anyhow!("error parsing module specifier: {:?}", e))
  }

  fn _parse(input: &str) -> IResult<&str, Self> {
    let (input, _) = tag("appkit:///@appkit/")(input)?;
    let (input, submodule) = alt((take_till(|b| b == '/'), eof))(input)?;

    match submodule {
      "env" => Ok((input, ModuleSource::Env)),
      "widgets" => Self::_parse_widget_source(input),
      _ => Ok((input, ModuleSource::Unknown)),
    }
  }

  fn _parse_widget_source(input: &str) -> IResult<&str, Self> {
    // remove "/" at the beginning
    let (input, _) = tag("/")(input)?;
    let (input, widget_id) = take_till(|b| b == '/')(input)?;

    let (input, _) = tag("/")(input)?;
    let (input, field_name) = alt((take_till(|b| b == '/'), eof))(input)?;

    Ok((
      input,
      ModuleSource::WidgetQuery(WidgetQuerySource {
        widget_id: widget_id.to_owned(),
        field_name: field_name.to_owned(),
      }),
    ))
  }
}
