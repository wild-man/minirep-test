use std::sync::Arc;

use logs::{LevelFilter, Logs, info};
use ntex::web::{self, HttpResponse};
use ntex::{http, time::Seconds, util::PoolId};

mod handlers;

struct AppDI {
    db: sqlite::Connection,
}

unsafe impl Send for AppDI {}
unsafe impl Sync for AppDI {}

fn init_logger() {
    Logs::new()
        .level(LevelFilter::Info) // Info
        .target(env!("CARGO_PKG_NAME"))
        .init();
}
/*
insert into reports values ('worker-1', 'us', 10.5, 7, current_timestamp);
insert into reports values ('worker-2', 'us', 0.5, 4, current_timestamp);
insert into reports values ('worker-1', 'eu', 100.5, 3, current_timestamp);
insert into reports values ('worker-2', 'eu', 333.5, 1000, current_timestamp);
insert into reports values ('worker-3', 'eu', 9.5, 2, current_timestamp);

 */
fn build_app_di() -> Arc<AppDI> {
    let connection = sqlite::open(":memory:").expect("Failed to open SQLite database");
    let query = "
        CREATE TABLE reports (worker_id TEXT, pool TEXT, hashrate REAL, temperature INTEGER, timestamp INTEGER);
        CREATE INDEX 'idx_reports_timestamp' on reports(timestamp);
    ";

    connection.execute(query).expect("Failed init sqlite");
    Arc::new(AppDI { db: connection })
}

async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().finish()
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    let di = build_app_di();

    info!("Started http server: 0.0.0.0:8000");
    ntex::server::build()
        .backlog(1024)
        .bind("localhost", "0.0.0.0:8000", move |cfg| {
            cfg.memory_pool(PoolId::P1);
            PoolId::P1.set_read_params(655350, 2048);
            PoolId::P1.set_write_params(655350, 2048);

            http::HttpService::build()
                // .keep_alive(http::KeepAlive::Timeout(ntex::time::Seconds(30)))
                .keep_alive(http::KeepAlive::Os)
                .client_timeout(Seconds::new(15))
                .headers_read_rate(Seconds::new(1), Seconds::new(3), 0)
                .payload_read_rate(Seconds::new(10), Seconds::new(20), 0)
                .h1(web::App::new()
                    .state(di.clone())
                    //                    .wrap(web::middleware::Logger::default())
                    .route("/stats", web::get().to(handlers::get_stats::http_handler))
                    .route(
                        "/report",
                        web::post().to(handlers::post_report::http_handler),
                    )
                    .route("", web::get().to(not_found))
                    .route("", web::post().to(not_found)))
        })
        .expect("Failed to bind to port")
        .workers(num_cpus::get())
        .run()
        .await
}
