use anyhow::{bail, Result};
use nom::bytes::complete::{tag, take_until};
use nom::combinator::eof;
use nom::multi::many0;
use nom::sequence::{pair, terminated};
use nom::IResult;

#[derive(Debug)]
pub struct Template {
  pub id: String,
  pub version: String,
}

pub fn parse<'a>(uri: &'a str) -> Result<Template> {
  let (input, template) =
    parse_template_uri(uri).expect("invalid template uri");
  if input.is_empty() {
    return Ok(template);
  }
  bail!("Invalid template uri");
}

fn parse_template_uri<'a>(uri: &'a str) -> IResult<&'a str, Template> {
  let (input, app_id_segment) =
    many0(terminated(take_until("/"), tag("/")))(uri)?;
  let (input, version) = terminated(
    many0(terminated(take_until("."), tag("."))),
    pair(tag("js"), eof),
  )(input)?;

  Ok((
    input,
    Template {
      id: app_id_segment.join("/"),
      version: version.join("."),
    },
  ))
}
