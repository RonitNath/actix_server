use std::collections::HashMap;

use actix_web::{
    body::BoxBody,
    dev::ServiceResponse,
    error,
    http::{ header::ContentType, StatusCode },
    middleware::{ ErrorHandlerResponse, ErrorHandlers },
    web,
    Error,
    HttpResponse,
    Responder,
    Result, get,
};
use tera::Tera;

use log::error;

pub mod routes {
    pub use super::{index, about};
}

// query: web::Query<HashMap<String, String>>

fn render_template_with_vars(tmpl: &Tera, name: &str, vars: HashMap<&str, &str>) -> Result<String> {
    let mut ctx = tera::Context::new();
    for (k, v) in vars {
        ctx.insert(k, &v);
    }
    tmpl.render(&[name, ".html.tera"].join(""), &ctx).map_err(|e| {
        error!("{e}");
        error::ErrorInternalServerError("Internal Server Error - Please try again later.")
    })
}

#[get("/")]
pub async fn index(
    tmpl: web::Data<tera::Tera>,
) -> Result<impl Responder, Error> {
    Ok(HttpResponse::Ok().body(render_template_with_vars(&tmpl, "index", HashMap::from([("title", "Home")]))?))
}

#[get("/about")]
pub async fn about(
    tmpl: web::Data<tera::Tera>,
) -> Result<impl Responder, Error> {
    Ok(HttpResponse::Ok().body(render_template_with_vars(&tmpl, "about", HashMap::from([("title", "About")]))?))
}

// Custom error handlers, to return HTML responses when an error occurs.
pub fn error_handlers() -> ErrorHandlers<BoxBody> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
pub fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<BoxBody>> {
    let response = get_error_response(&res, "Page not found");
    Ok(
        ErrorHandlerResponse::Response(
            ServiceResponse::new(res.into_parts().0, response.map_into_left_body())
        )
    )
}

// Generic error handler.
pub fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> HttpResponse {
    let request = res.request();

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |err: &str| {
        HttpResponse::build(res.status())
            .content_type(ContentType::plaintext())
            .body(err.to_string())
    };

    let tera = request.app_data::<web::Data<Tera>>().map(|t| t.get_ref());
    match tera {
        Some(tera) => {
            let mut context = tera::Context::new();
            context.insert("error", error);
            context.insert("status_code", res.status().as_str());
            let body = tera.render("error/404.html.tera", &context);

            match body {
                Ok(body) =>
                    HttpResponse::build(res.status()).content_type(ContentType::html()).body(body),
                Err(_) => fallback(error),
            }
        }
        None => fallback(error),
    }
}
