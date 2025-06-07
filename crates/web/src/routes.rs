use actix_web::{get, HttpResponse};

#[get("/health_check")]
pub async fn health_check() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().finish())
}

#[get("/")]
pub async fn index() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().finish())
}

#[get("/qr")]
pub async fn qr() -> Result<HttpResponse, actix_web::Error> {
    let name = "Alice";
    let single = "yes";
    let is_verified = true; // Change to false to test not verified

    // Determine verification status HTML
    let verify_html = if is_verified {
        r#"<div class="verify-status verified">Verified ✔</div>"#
    } else {
        r#"<div class="verify-status not-verified">Not Verified ✖</div>"#
    };

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>User Profile</title>
            <style>
                body {{
                    font-family: Arial, sans-serif;
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    height: 100vh;
                    margin: 0;
                    background-color: #f0f2f5;
                }}
                .profile-card {{
                    background-color: white;
                    border-radius: 8px;
                    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
                    padding: 20px;
                    width: 300px;
                    text-align: center;
                }}
                .profile-card h2 {{
                    margin: 0 0 10px;
                    color: #333;
                }}
                .profile-card p {{
                    margin: 5px 0;
                    color: #666;
                    font-size: 16px;
                }}
                .verify-status {{
                    margin-top: 15px;
                    padding: 8px;
                    border-radius: 5px;
                    font-size: 14px;
                    font-weight: bold;
                }}
                .verified {{
                    background-color: #e6f3e6;
                    color: #2e7d32;
                }}
                .not-verified {{
                    background-color: #ffe6e6;
                    color: #d32f2f;
                }}
            </style>
        </head>
        <body>
            <div class="profile-card">
                <h2>{}</h2>
                <p>Single: {}</p>
                {}
            </div>
        </body>
        </html>
        "#,
        name, single, verify_html
    );
    // Return HTML response with correct content type
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
