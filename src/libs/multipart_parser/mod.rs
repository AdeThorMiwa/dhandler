use axum::extract::{multipart::MultipartError, Multipart};
use bytes::Bytes;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Cursor;
use std::pin::Pin;
use tokio::io::AsyncRead;
use tracing::instrument;

pub type FileStream = Pin<Box<dyn AsyncRead + Send>>;

pub struct ParsedMultipart<B> {
    pub body: B,
    pub files: HashMap<String, FileStream>,
}

impl<B: Debug> Debug for ParsedMultipart<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParsedMultipart")
            .field("body", &self.body)
            .field("files", &format!("[{} files]", self.files.len()))
            .finish()
    }
}

pub enum ParseMultipartError {
    MissingRequiredFiles,
    SerializationError(serde_json::Error),
    MultipartError(MultipartError),
}

impl From<ParseMultipartError> for loco_rs::Error {
    fn from(err: ParseMultipartError) -> Self {
        match err {
            ParseMultipartError::MissingRequiredFiles => {
                tracing::error!("missing required files");
                Self::BadRequest("Missing required files".to_string())
            }
            ParseMultipartError::SerializationError(e) => {
                tracing::error!("Serialiazation error: {e}");
                Self::InternalServerError
            }
            ParseMultipartError::MultipartError(e) => {
                tracing::error!("Multipart error: {e}");
                Self::InternalServerError
            }
        }
    }
}

/// Parses a multipart request into a `ParsedMultipart` struct.
///
/// # Errors
///
/// Returns a `ParseMultipartError` if the multipart request is invalid.
#[instrument(skip(multipart))]
pub async fn parse_multipart<B>(
    mut multipart: Multipart,
    file_list: Vec<&str>,
) -> Result<ParsedMultipart<B>, ParseMultipartError>
where
    B: for<'de> serde::Deserialize<'de>,
{
    let mut fields: HashMap<String, String> = HashMap::new();
    let mut files = HashMap::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(ParseMultipartError::MultipartError)?
    {
        let name = field.name().unwrap_or_default().to_string();

        if field.file_name().is_some() {
            if file_list.contains(&name.as_str()) {
                let bytes: Bytes = field
                    .bytes()
                    .await
                    .map_err(ParseMultipartError::MultipartError)?;

                let reader: Pin<Box<dyn AsyncRead + Send>> = Box::pin(Cursor::new(bytes));

                files.insert(name, reader);
            }
        } else {
            let value = field
                .text()
                .await
                .map_err(ParseMultipartError::MultipartError)?;

            fields.insert(name, value);
        }
    }

    if file_list.len() != files.len() {
        return Err(ParseMultipartError::MissingRequiredFiles);
    }

    let fields = serde_json::to_string(&fields).map_err(ParseMultipartError::SerializationError)?;

    let body =
        serde_json::from_str::<B>(&fields).map_err(ParseMultipartError::SerializationError)?;

    Ok(ParsedMultipart { body, files })
}
