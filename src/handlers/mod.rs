use logs::error;

use ntex::web::HttpResponse;

pub mod get_stats;
pub mod post_report;

pub fn internal_error(err: String) -> HttpResponse {
    error!("Error: {}", err);
    HttpResponse::InternalServerError().finish()
}

pub fn bad_request_error(err: String) -> HttpResponse {
    error!("Error: {}", err);
    HttpResponse::BadRequest().finish()
}
