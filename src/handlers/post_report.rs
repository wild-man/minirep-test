use logs::error;
use sqlite::Value as SqliteValue;
use std::{sync::Arc, time::SystemTime};

use ntex::web::{self, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AppDI,
    handlers::{bad_request_error, internal_error},
};

#[derive(Deserialize, Debug)]
struct Report {
    worker_id: String,
    pool: String,
    hashrate: f64,
    temperature: i64,
    timestamp: u64,
}

pub async fn http_handler(
    _req: HttpRequest,
    payload: web::types::Json<Value>,
    app: web::types::State<Arc<AppDI>>,
) -> HttpResponse {
    let report = payload.into_inner();

    let report = match serde_json::from_value::<Report>(report) {
        Ok(report) => report,
        Err(err) => {
            error!("Failed to deserialize report: {}", err);
            return HttpResponse::BadRequest().finish();
        }
    };

    dbg!(&report);

    let curent_timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if report.timestamp > curent_timestamp || report.timestamp < (curent_timestamp - 300) {
        error!(
            "bad report timestamp: {}; current timestamp: {}",
            report.timestamp, curent_timestamp
        );
        return HttpResponse::BadRequest().finish();
    }

    dbg!(&curent_timestamp);

    let query = "insert into reports (worker_id, pool, hashrate, temperature, timestamp) values (:worker_id, :pool, :hashrate, :temperature, :timestamp)";
    let statement = app.db.prepare(query);

    if let Err(err) = statement {
        return bad_request_error(err.to_string());
    }

    let mut st = statement.unwrap();

    let bind = st.bind_iter::<_, (_, SqliteValue)>([
        (":worker_id", report.worker_id.into()),
        (":pool", report.pool.into()),
        (":hashrate", report.hashrate.into()),
        (":temperature", report.temperature.into()),
        (":timestamp", (report.timestamp as i64).into()),
    ]);

    if let Err(err) = bind {
        return internal_error(err.to_string());
    }

    let inserted = st.next();

    if let Err(err) = inserted {
        return internal_error(err.to_string());
    }

    HttpResponse::Created().finish()
}
