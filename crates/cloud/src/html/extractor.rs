use anyhow::anyhow;
use html5ever::tokenizer::{TagKind, Token, TokenSink, TokenSinkResult};
use html5ever::LocalName;

macro_rules! log {
  ($should_log:expr,$($arg:tt)*) => {{
    if $should_log {
      print!($($arg)*);
    }
  }};
}

macro_rules! bail {
  ($($arg:tt)*) => {{
    return TokenSinkResult::Script(ProcessResult::Error(anyhow!($($arg)*)));
  }};
}

#[derive(Clone)]
pub struct TextExtractor {
  debug: bool,
  ignore_tags: Vec<String>,
  /// Stack of tags that were opened but hasn't closed
  tags: Vec<LocalName>,
  in_char_run: bool,
  curr_text: Vec<char>,
  /// A tuple of (tag name, text)
  texts: Vec<(String, String)>,
}

#[derive(Default, Debug, Clone)]
pub struct TextExtractorOptions {
  pub debug: bool,

  /// Tags to ignore when extracting text; for example, "style", "script", etc
  pub ignore_tags: Vec<String>,
}

impl TextExtractor {
  pub fn new(options: TextExtractorOptions) -> Self {
    Self {
      debug: options.debug,
      ignore_tags: options.ignore_tags,
      tags: vec![],
      in_char_run: false,
      curr_text: vec![],
      texts: vec![],
    }
  }

  pub fn get_texts(&self) -> &Vec<(String, String)> {
    self.texts.as_ref()
  }

  fn is_char(&mut self, is_char: bool) {
    match (self.in_char_run, is_char) {
      (false, true) => {
        log!(self.debug, "CHAR : \"")
      }
      (true, false) => {
        if !self
          .tags
          .iter()
          .any(|tag| self.ignore_tags.iter().any(|ig| tag.eq(ig)))
        {
          self.texts.push((
            self
              .tags
              .last()
              .map(|t| t.to_string())
              .unwrap_or("".to_owned()),
            self.curr_text.iter().collect(),
          ));
        }
        self.curr_text = vec![];
        log!(self.debug, "\"\n")
      }
      _ => (),
    }
    self.in_char_run = is_char;
  }

  fn do_char(&mut self, c: char) {
    self.is_char(true);
    self.curr_text.push(c);
    log!(self.debug, "{}", c.escape_default().collect::<String>());
  }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ProcessResult {
  Ok,
  Error(anyhow::Error),
}

impl TokenSink for TextExtractor {
  type Handle = ProcessResult;

  fn process_token(
    &mut self,
    token: Token,
    _line_number: u64,
  ) -> TokenSinkResult<ProcessResult> {
    match token {
      Token::CharacterTokens(b) => {
        for c in b.chars() {
          self.do_char(c);
        }
      }
      Token::NullCharacterToken => self.do_char('\0'),
      Token::TagToken(tag) => {
        self.is_char(false);
        // This is not proper HTML serialization, of course.
        match tag.kind {
          TagKind::StartTag => {
            self.tags.push(tag.name.clone());
            log!(self.debug, "TAG  : <\x1b[32m{}\x1b[0m", tag.name)
          }
          TagKind::EndTag => {
            if self.tags.last().map(|t| t != &tag.name).unwrap_or(false) {
              bail!("Tag {:?} is not closed", self.tags.last().unwrap());
            }
            self.tags.pop();
            log!(self.debug, "TAG  : <\x1b[31m/{}\x1b[0m", tag.name)
          }
        }
        for attr in tag.attrs.iter() {
          log!(
            self.debug,
            " \x1b[36m{}\x1b[0m='\x1b[34m{}\x1b[0m'",
            attr.name.local,
            attr.value
          );
        }
        if tag.self_closing {
          log!(self.debug, " \x1b[31m/\x1b[0m");
        }
        log!(self.debug, ">\n");
      }
      Token::ParseError(err) => {
        self.is_char(false);
        log!(self.debug, "ERROR: {}\n", err);
        bail!("Error: {}", err);
      }
      Token::EOFToken => {
        self.is_char(false);
        log!(self.debug, "EOF\n");
      }
      _ => {
        self.is_char(false);
        log!(self.debug, "OTHER: {:?}\n", token);
        bail!("Unknown token: {:?}", token);
      }
    }
    TokenSinkResult::Continue
  }
}
