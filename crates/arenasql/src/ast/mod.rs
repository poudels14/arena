mod datatype;

pub mod statement;
pub use datatype::cast_unsupported_data_types;

use sqlparser::ast::{DataType, Expr, Statement as SQLStatement, StructField};
use sqlparser::dialect::{Dialect, PostgreSqlDialect};
use sqlparser::keywords::Keyword;
use sqlparser::parser::{Parser, ParserError, ParserOptions};
use sqlparser::tokenizer::{Token, TokenWithLocation, Tokenizer};
use tracing::debug;

use crate::Result as ArenasqlResult;

pub fn parse(sql: &str) -> ArenasqlResult<Vec<SQLStatement>> {
  Ok(ArenasqlParser::parse_sql(&PostgreSqlDialect {}, sql)?)
}

/// Parses the query and "sanitizes" the statements so that they
/// can run in Datafusion.
/// The sanitizer updates the statements to support features like
/// JSONB, VECTOR and other datatype
pub fn sanitize(statements: &mut Vec<SQLStatement>) -> ArenasqlResult<()> {
  statements
    .iter_mut()
    .map(|stmt| cast_unsupported_data_types(stmt))
    .collect::<ArenasqlResult<()>>()
}

struct ArenasqlParser<'a> {
  dialect: &'a dyn Dialect,
  parser: Parser<'a>,
  options: ParserOptions,
}

impl<'a> ArenasqlParser<'a> {
  pub fn parse_sql(
    dialect: &'a dyn Dialect,
    sql: &str,
  ) -> Result<Vec<SQLStatement>, ParserError> {
    Self {
      dialect,
      parser: Parser::new(dialect),
      options: ParserOptions::default(),
    }
    .try_with_sql(sql)?
    .parse_statements()
  }

  pub fn try_with_sql(self, sql: &str) -> Result<Self, ParserError> {
    debug!("Parsing sql '{}'...", sql);
    let tokens = Tokenizer::new(self.dialect, sql)
      .with_unescape(self.options.unescape)
      .tokenize_with_location()?;
    Ok(self.with_tokens_with_locations(tokens))
  }

  pub fn with_tokens_with_locations(
    mut self,
    tokens: Vec<TokenWithLocation>,
  ) -> Self {
    self.parser = self.parser.with_tokens_with_locations(tokens);
    self
  }

  pub fn parse_statements(&mut self) -> Result<Vec<SQLStatement>, ParserError> {
    let mut stmts = Vec::new();
    let mut expecting_statement_delimiter = false;
    loop {
      // ignore empty statements (between successive statement delimiters)
      while self.parser.consume_token(&Token::SemiColon) {
        expecting_statement_delimiter = false;
      }

      match self.parser.peek_token().token {
        Token::EOF => break,
        // end of statement
        Token::Word(word) if word.keyword == Keyword::END => break,
        _ => {}
      }

      if expecting_statement_delimiter {
        return self
          .parser
          .expected("end of statement", self.parser.peek_token());
      }

      let mut statement = self.parser.parse_statement()?;
      if let SQLStatement::CreateIndex {
        ref mut predicate, ..
      } = statement
      {
        if self.parser.parse_keyword(Keyword::WITH) {
          self.parser.prev_token();
          let with_options = self.parser.parse_options(Keyword::WITH)?;
          let (values, fields) = with_options
            .into_iter()
            .map(|op| {
              (
                Expr::Value(op.value),
                StructField {
                  field_name: Some(op.name.clone()),
                  // this field type can be anything since data type can
                  // be derived from Value
                  field_type: DataType::Text,
                },
              )
            })
            .unzip();
          *predicate = Some(Expr::Struct { values, fields });
        }
      }
      stmts.push(statement);
      expecting_statement_delimiter = true;
    }
    Ok(stmts)
  }
}
