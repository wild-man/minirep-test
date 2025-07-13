use serde::Serialize;
use sqlite::{State, Statement};
use std::sync::Arc;

use ntex::web::{self, HttpRequest, HttpResponse};

use crate::{AppDI, handlers::internal_error};
// CREATE TABLE reports (worker_id TEXT, pool TEXT, hashrate REAL, temperature INTEGER, timestamp INTEGER);

#[derive(Serialize, Debug)]
struct AvgRow {
    pool: String,
    worker_count: i64,
    avg_hashrate: f64,
    avg_temp: f64,
}

fn db_row(statement: &Statement<'_>) -> Result<AvgRow, String> {
    let pool = statement
        .read::<String, _>("pool")
        .map_err(|e| e.to_string())?;

    let worker_count = statement
        .read::<i64, _>("worker_count")
        .map_err(|e| e.to_string())?;

    let avg_hashrate = statement
        .read::<f64, _>("avg_hashrate")
        .map_err(|e| e.to_string())?;

    let avg_temp = statement
        .read::<f64, _>("avg_temp")
        .map_err(|e| e.to_string())?;

    let row = AvgRow {
        pool,
        worker_count,
        avg_hashrate,
        avg_temp,
    };
    dbg!(&row);
    Ok(row)
}

pub async fn http_handler(_req: HttpRequest, app: web::types::State<Arc<AppDI>>) -> HttpResponse {
    let query = "
    select
        pool,
        count(distinct worker_id) as worker_count,
        avg(hashrate) as avg_hashrate,
        avg(temperature) as avg_temp

    from reports
    where timestamp > (unixepoch() - 10)
    group by pool";

    let statement = app.db.prepare(query);

    if let Err(err) = statement {
        return internal_error(err.to_string());
    }

    let mut statement = statement.unwrap();
    let mut avg: Vec<AvgRow> = Vec::new();

    while let Ok(State::Row) = statement.next() {
        let r = db_row(&statement);

        if let Err(err) = r {
            return internal_error(err);
        }
        dbg!(&r);
        avg.push(r.unwrap());
    }
    //@todo delete old data with every 10-100 requests for example
    HttpResponse::Ok().json(&avg)
}
