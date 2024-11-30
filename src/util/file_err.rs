use crate::util::mime::get_mime_type;
use http::{header, StatusCode};
use pingora::http::ResponseHeader;
use pingora::prelude::Session;

pub static PAGE404: &str = r#"<!DOCTYPE html>
<html>
  <head>
    <title>404</title>
    <style>
      html {
        color-scheme: light dark;
      }
      body {
        width: 35em;
        margin: 0 auto;
        font-family: Tahoma, Verdana, Arial, sans-serif;
      }
    </style>
  </head>
  <body>
    <h1>404 Not Found</h1>
    <p>
      The page you visited does not exist.
    </p>
    <p><em>Powered by <a href="https://pingpong.bluemangoo.net/">Pingpong</a>.</em></p>
  </body>
</html>
"#;

pub static PAGE50X: &str = r#"<!DOCTYPE html>
<html>
  <head>
    <title>An error occurred.</title>
    <style>
      html {
        color-scheme: light dark;
      }
      body {
        width: 35em;
        margin: 0 auto;
        font-family: Tahoma, Verdana, Arial, sans-serif;
      }
    </style>
  </head>
  <body>
    <h1>An error occurred.</h1>
    <p>
      Sorry, the page you are looking for is currently unavailable.<br/>
      Please try again later.
    </p>
    <p><em>Powered by <a href="https://pingpong.bluemangoo.net/">Pingpong</a>.</em></p>
  </body>
</html>
"#;

pub async fn make_page50x(session: &mut Session, status:StatusCode) -> pingora::Result<bool> {
    let content_length = PAGE50X.len();
    let mut resp = ResponseHeader::build(status, Some(4))?;
    resp.insert_header(header::SERVER, "Pingpong")?;
    resp.insert_header(header::CONTENT_LENGTH, content_length.to_string())?;
    resp.insert_header(header::CONTENT_TYPE, get_mime_type(".html"))?;

    session.write_response_header(Box::new(resp), false).await?;

    session
        .write_response_body(Some(PAGE50X.into()), true)
        .await?;
    Ok(true)
}
