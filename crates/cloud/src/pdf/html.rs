use anyhow::{Context, Result};
use common::dirs;
use common::downloader;
use pdfium_render::prelude::*;
use runtime::deno::core::{op2, JsBuffer, OpState};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfToHtmlOptions {
  pdfium_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPage {
  html: String,
}

/// Returns a list of PdfPage, one for each page in the pdf document
#[op2(async)]
#[serde]
pub async fn op_cloud_pdf_to_html(
  _state: Rc<RefCell<OpState>>,
  #[buffer] pdf_bytes: JsBuffer,
  #[serde] options: PdfToHtmlOptions,
) -> Result<Vec<PdfPage>> {
  let lib_path = get_pdfium_lib_path().await?.canonicalize()?;
  let pdfium = Pdfium::new(
    Pdfium::bind_to_library(
      &options
        .pdfium_path
        .unwrap_or(lib_path.to_string_lossy().to_string()),
    )
    .or_else(|_| Pdfium::bind_to_system_library())?,
  );

  let document = pdfium
    .load_pdf_from_byte_vec(pdf_bytes.to_vec(), None)
    .with_context(|| "Failed to load pdf")?;

  Ok(document
    .pages()
    .iter()
    .map(|page| {
        let page_width = page.width().value;
        let page_height = page.height().value;
        let page_texts = page
            .objects()
            .iter()
            .filter_map(|object| {
                object.as_text_object().and_then(|object| {
                    if object.text().len() == 0 {
                      return  None;
                    }

                    let transform = object.matrix().unwrap();
                    // Note(sagar): use percentage for positioning
                    let top = 100 as f32 - (transform.f() * 100 as f32) / page_height;
                    let left = (transform.e() * 100 as f32) / page_width;
                    let mut font_size =  object.unscaled_font_size().value;
                    if font_size < 1.0 {
                      // sometimes, the font size is wrong for some reason
                      // TODO(sagar): find a better way to determine font size
                      // It seems like the font size is incorrect when the font name
                      // is empty. is it due to issue with loading fonts?
                      font_size = 2.43 / font_size;
                    }

                    let rotation = object.get_rotation_clockwise_degrees();
                    let text = object.text();
                    let text = html_escape::encode_text(&text);

                    Some(format!(
                      r#"<div class="pdf-text" style="position:absolute; left: {left:.2}%; top: {top:.2}%; font-size: calc(var(--scale-factor)*{font_size:.2}px); white-space: pre; transform-origin: 0 0; transform: scaleX(1) rotate({rotation}deg)">{text}</div>
                  "#
                  ))
                })
            })
            .collect::<Vec<_>>()
            .join("");

        let html = format!(
          r#"<div class="arena-pdf-page" style="position: relative; margin: auto;
          width: calc(var(--scale-factor)*{page_width}px);
          height: calc(var(--scale-factor)*{page_height}px);
          text-wrap: nowrap;
          transform: scale(1.0);
          ">{page_texts}</div>"#);

        PdfPage {
          html
        }
    })
    .collect::<Vec<_>>()
  )
}

async fn get_pdfium_lib_path() -> Result<PathBuf> {
  let pdfium_dir = dirs::portal()?.cache_dir().join("pdfium");
  let lib_path = pdfium_dir.join("lib/libpdfium.so");
  if !lib_path.exists() {
    let os = match env!("CARGO_CFG_TARGET_OS") {
      "linux" => "linux",
      "macos" => "mac",
      os => panic!("Unsupported OS: {:?}", os),
    };
    let arch = match env!("CARGO_CFG_TARGET_ARCH") {
      "x86_64" => "x64",
      "x86" => "x86",
      "arm" => "arm",
      "aarch64" => "arm64",
      arch => panic!("Unsupported architecture: {:?}", arch),
    };
    downloader::download_and_extract_tgz(
      &format!(
        "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-{}-{}.tgz",
        os,
        arch,
        ),
      &pdfium_dir
    )
    .await?;
  }
  Ok(lib_path)
}
