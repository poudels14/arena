use cloud::html::extractor::{TextExtractor, TextExtractorOptions};
use html5ever::tendril::*;
use html5ever::tokenizer::{BufferQueue, TokenizerResult};
use html5ever::tokenizer::{Tokenizer, TokenizerOpts};

fn main() {
  let extractor = TextExtractor::new(TextExtractorOptions {
    debug: true,
    ..Default::default()
  });

  let chunk: Tendril<_> =
    r#"<div class="test-cls">nice<span>yo!</span></div>"#.to_tendril();
  let mut input = BufferQueue::new();
  input.push_back(chunk.try_reinterpret().unwrap());

  let mut tok = Tokenizer::new(
    extractor,
    TokenizerOpts {
      profile: true,
      ..Default::default()
    },
  );
  let res = tok.feed(&mut input);
  match res {
    TokenizerResult::Done => {
      println!("HTML parsed successfully");
    }
    TokenizerResult::Script(h) => {
      println!("Parsig failed = {:?}", h);
    }
  }
  tok.end();
  println!(
    "TEXTS: {}",
    tok
      .sink
      .get_texts()
      .iter()
      .map(|t| t.1.clone())
      .collect::<Vec<String>>()
      .join("+")
  );
}
