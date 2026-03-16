use std::{io, pin::Pin};
use tokio::io::{AsyncRead, AsyncReadExt};

use pdfium_render::prelude::*;

/// Reads a markdown document from a stream.
///
/// # Errors
///
/// This function will return an error if the underlying stream returns an error.
///
/// # Panics
///
/// This function will panic if the underlying stream returns an error.
pub async fn read_from_stream(buf: &mut Pin<Box<dyn AsyncRead + Send>>) -> std::io::Result<String> {
    let mut bytes = Vec::new();
    buf.read_to_end(&mut bytes).await?;

    tokio::task::spawn_blocking(move || {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
                &std::env::var("LIBPDFIUM_LIB_PATH")
                    .unwrap_or_else(|_| "/usr/local/lib".to_string()),
            ))
            .expect("libpdfium not found"),
        );

        let doc = pdfium
            .load_pdf_from_byte_vec(bytes, None)
            .map_err(io::Error::other)?;

        let mut markdown = String::new();
        for page in doc.pages().iter() {
            let text = page.text().map_err(io::Error::other)?.all();
            markdown.push_str(&text);
            markdown.push_str("\n\n");
        }

        Ok(markdown)
    })
    .await?
}
