use async_trait::async_trait;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{multispace0, newline};
use nom::sequence::{pair, terminated};
use nom::IResult;
use pgwire::api::stmt::QueryParser;
use pgwire::error::PgWireResult;
use sqlparser::ast::Statement as SQLStatement;

use crate::auth::AuthHeader;

#[derive(Debug, Clone)]
pub struct ArenaQuery {
  pub client: AuthHeader,
  pub stmts: Vec<Box<SQLStatement>>,
}

pub struct ArenaQueryParser;

#[async_trait]
impl QueryParser for ArenaQueryParser {
  type Statement = ArenaQuery;

  async fn parse_sql(
    &self,
    sql: &str,
    _types: &[pgwire::api::Type],
  ) -> PgWireResult<Self::Statement> {
    let (sql, header) =
      parse_auth_header(&sql).unwrap_or_else(|_| (sql, AuthHeader::None));

    let stmts = arenasql::ast::parse_and_sanitize(sql)?
      .into_iter()
      .map(|stmt| Box::new(stmt))
      .collect();

    Ok(ArenaQuery {
      client: header,
      stmts,
    })
  }
}

/// Parses the first line of the query to check for authorization header.
/// The first line should correspond to either a JWT authorization token
/// OR an id of the session that was created after the authorization.
///
///   -- X-ARENASQL-AUTH:base64_encoded_jwt_token()
///   -- X-ARENASQL-AUTH:session-{session-id}
///
/// Any query with the JWT authorization header will create a new session.
///
/// TODO: the server should somehow pass the info to the client if it decides
/// to close a session. This will happen if the server has too many sessions
/// active
fn parse_auth_header<'a>(header: &'a str) -> IResult<&'a str, AuthHeader> {
  let (remaining, _) = multispace0(header)?;
  let (remaining, _) = tag("-- X-ARENASQL-AUTH:")(remaining)?;
  let (remaining, header) = alt((parse_session_id, parse_jwt))(remaining)?;
  Ok((remaining, header))
}

fn parse_session_id<'a>(line: &'a str) -> IResult<&'a str, AuthHeader> {
  let (remaining, (_, session_id)) =
    pair(tag("session-"), terminated(take_until("\n"), newline))(line)?;
  Ok((
    remaining,
    AuthHeader::Authenticated {
      session_id: session_id.to_owned(),
    },
  ))
}

fn parse_jwt<'a>(line: &'a str) -> IResult<&'a str, AuthHeader> {
  let (remaining, (_, token)) =
    pair(tag("token-"), terminated(take_until("\n"), newline))(line)?;
  Ok((
    remaining,
    AuthHeader::Token {
      token: token.to_owned(),
    },
  ))
}

#[cfg(test)]
mod tests {
  use super::parse_auth_header;
  use crate::auth::AuthHeader;

  #[test]
  fn parser_test_parse_session_id() {
    let query = "-- X-ARENASQL-AUTH:session-some-id\nSELECT * FROM users;";
    let (_, session_header) = parse_auth_header(query).unwrap();

    assert_eq!(
      session_header,
      AuthHeader::Authenticated {
        session_id: "some-id".to_owned()
      }
    );
  }

  #[test]
  fn parser_test_parse_jwt_token() {
    let token = "eyJ1c2VyIjogIm5ldy11c2VyIiwgImRhdGFiYXNlIjogInRlc3QtZGIifQ==";
    let (_, session_header) = parse_auth_header(&format!(
      "-- X-ARENASQL-AUTH:token-{}\nSELECT * FROM users;",
      token
    ))
    .unwrap();

    assert_eq!(
      session_header,
      AuthHeader::Token {
        token: token.to_owned()
      }
    );
  }

  #[test]
  fn parser_test_parse_invalie_header() {
    let res = parse_auth_header(
      // there's no space after first "--"
      "--X-ARENASQL-AUTH:token-some-base64-token\nSELECT * FROM users;",
    );

    assert!(
      res.is_err(),
      "Expected to return error when space is missing after '--'"
    );

    let res = parse_auth_header(
      // there's no "token-" prefix infront of auth token
      "-- X-ARENASQL-AUTH:some-base64-token\nSELECT * FROM users;",
    );

    assert!(
      res.is_err(),
      "Expected to return error when 'token-' is missing"
    );
  }
}
