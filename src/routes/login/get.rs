use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn login_form(flash_message: IncomingFlashMessages) -> HttpResponse {
    let mut err_html = String::new();
    for m in flash_message.iter() {
        writeln!(err_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
<!DOCTYPE html>
<html lang="en">

<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Login</title>
</head>

<body style="display: flex; align-items: center; justify-content: center; flex-direction: column; min-height:100vh">
{err_html}
<form action="/login" method="post" style="padding: 10px; border: 1px solid blueviolet;">
        <label>Username
            <input type="text" placeholder="Enter Username" name="username">
        </label>
        <label>Password
            <input type="password" placeholder="Enter Password" name="password">
        </label>
        <button type="submit">Login</button>
    </form>
</body>

</html>
            "#
        ))
}
