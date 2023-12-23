use anyhow::{anyhow, Context, Result};
use pdfium_render::prelude::*;
use runtime::deno::core::{op2, JsBuffer, OpState};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfToHtmlOptions {
  /// Uses `~/.arena/pdfium/libpdfium.so` by default
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
  let pdfium = Pdfium::new(
    Pdfium::bind_to_library(
      &options.pdfium_path.unwrap_or(
        dirs::home_dir()
          .ok_or(anyhow!("Failed to find HOME directory"))?
          .join(".arena/pdfium/libpdfium.so")
          .to_string_lossy()
          .to_string(),
      ),
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
