use std::{io::Cursor, marker::PhantomData, path::PathBuf};

use rocket::{
  http::{ContentType, Method, Status},
  route::{Handler, Outcome},
  Data, Request, Response, Route,
};
use rust_embed::RustEmbed;

pub struct EmbedServer<T: RustEmbed> {
  _phantom: PhantomData<T>,
}

impl<T: RustEmbed> Clone for EmbedServer<T> {
  fn clone(&self) -> Self {
    Self { _phantom: PhantomData }
  }
}

impl<T: RustEmbed + 'static + Send + Sync> EmbedServer<T> {
  pub fn new() -> Self {
    Self { _phantom: PhantomData }
  }
}

impl<T: RustEmbed + 'static + Send + Sync> Into<Vec<Route>> for EmbedServer<T> {
  fn into(self) -> Vec<Route> {
    vec![Route::ranked(-2, Method::Get, "/<path..>", self.clone())]
  }
}

#[async_trait::async_trait]
impl<T: RustEmbed + 'static + Send + Sync> Handler for EmbedServer<T> {
  async fn handle<'r>(&self, request: &'r Request<'_>, _data: Data<'r>) -> Outcome<'r> {
    let path: Option<PathBuf> = request.segments(0..).ok();

    match path {
      Some(path) => {
        let mut file = T::get(&path.to_string_lossy());
        let mut content_type = path
          .extension()
          .map(|x| x.to_string_lossy())
          .and_then(|x| ContentType::from_extension(&x))
          .unwrap_or(ContentType::Plain);

        if file.is_none() {
          file = T::get("index.html");
          content_type = ContentType::HTML;
        }

        match file {
          Some(buf) => {
            let response = Response::build()
              .header(content_type)
              .sized_body(buf.len(), Cursor::new(buf.into_owned()))
              .finalize();

            Outcome::Success(response)
          }
          None => Outcome::Failure(Status::NotFound),
        }
      }
      None => Outcome::Failure(Status::NotFound),
    }
  }
}
