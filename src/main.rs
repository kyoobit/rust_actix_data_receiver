use std::env;
use std::str;

// A web framework for Rust
// https://docs.rs/actix-web/latest/actix_web/web/index.html
// cargo add actix-web
use actix_web::{
    get, middleware::Logger, put, web, App, HttpResponse, HttpServer, Responder, Result,
};

// A Prometheus instrumentation middleware for use with actix-web
// https://docs.rs/actix-web-prom/latest/actix_web_prom/
// cargo add actix-web-prom
use actix_web_prom::PrometheusMetricsBuilder;

// Timezone-aware date and time
// https://docs.rs/chrono/latest/chrono/
// cargo add chrono
use chrono::{DateTime, Utc};

// Command Line Argument Parser for Rust
// https://docs.rs/clap/latest/clap/
// cargo add clap --features derive
use clap::Parser;

// A simple logger
// https://docs.rs/log/latest/log/
// https://docs.rs/actix-web/latest/actix_web/middleware/struct.Logger.html
// https://docs.rs/env_logger/latest/env_logger/
// cargo add env_logger
//use env_logger; // <--- this import is redundant

// https://docs.rs/rusqlite/latest/rusqlite
// cargo add rusqlite
use rusqlite::Connection;

// https://docs.rs/serde/latest/serde/
// https://serde.rs
// cargo add serde --features derive
use serde::{Deserialize, Serialize};

// A framework for instrumenting Rust
// https://docs.rs/tracing/latest/tracing
// cargo add tracing
// Utilities for implementing and composing tracing subscribers
// https://docs.rs/tracing-subscriber/latest/tracing_subscriber
// cargo add tracing-subscriber
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;

// TODO: DELETE /<database name>/<table name>/<key>
// TODO: GET /<database name>/<table name>/<key>
// TODO: PATCH /<database name>/<table name>/<key>

/// Create data in a database table using JSON formatted data
/// PUT /<database name>/<table name>
/// curl -i -X PUT -d '{"curl test": true}' http://localhost:8888/database/test
#[put("/{database_name}/{table_name}")]
async fn create_data(
    appdata: web::Data<AppData>, // Provide access to options entered on the CLI
    path: web::Path<(String, String)>, // Provide access to the URI path elements
    body: web::Bytes,            // Provide access to the request body
) -> Result<impl Responder> {
    // Validate the database name is sane
    // /{database_name <--- path.0}/{table_name <--- path.1}
    let database_files = appdata.database_files.to_string();
    let database_name = path.0.to_string();
    let database = format!("{database_files}/{database_name}.db");

    // Validate the table name is sane
    // /{database_name <--- path.0}/{table_name <--- path.1}
    let table_name = path.1.to_string();

    // Get a handle to the database
    // The database will be created as needed
    let conn = Connection::open(database).unwrap();

    // Create the table if it doesn't exist
    let sql_create_table = format!(
        "CREATE TABLE IF NOT EXISTS {table_name} (
            id INTEGER PRIMARY KEY,
            timestamp DATETIME NOT NULL,
            data TEXT NOT NULL
        );"
    );
    let result = conn.execute(&sql_create_table.to_string(), ()).unwrap();
    debug!("create result: {}", result);

    // Get the JSON data from the request
    let data = match str::from_utf8(&body) {
        Ok(data) => data,
        Err(_) => return Ok(HttpResponse::BadRequest()),
    };

    // Set the timestamp to the current time
    let timestamp: DateTime<Utc> = Utc::now();

    // Insert the data into the table
    // https://www.sqlite.org/about.html
    // https://www.sqlite.org/lang.html
    // https://www.sqlite.org/json1.html
    info!("insert timestamp: {timestamp}, data: {data}");
    let sql_insert = format!(
        "INSERT INTO {table_name} (timestamp, data)
        VALUES (:timestamp, json(:data));"
    );
    let result = conn
        .execute(
            &sql_insert.to_string(),
            &[
                (":timestamp", &timestamp.to_string()),
                (":data", &data.to_string()),
            ],
        )
        .unwrap();
    debug!("insert result: {}", result);

    // Return an HTTP 201 Created response
    Ok(HttpResponse::Created())
}

// Pong response structure
#[derive(Debug, Deserialize, Serialize)]
struct PongResponse {
    ping: String,
}

// Ping/Pong response handler
#[get("/ping")]
async fn ping() -> Result<impl Responder> {
    // Respond with a pong response as a sanity check
    let result = PongResponse {
        ping: String::from("pong"),
    };

    // Format the result into JSON
    // https://docs.rs/actix-web/latest/actix_web/web/struct.Json.html
    Ok(web::Json(result))
}

// Application data passed to endpoints
struct AppData {
    database_files: String,
}

// Get a environment variable's value
fn get_env_var(key: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_) => String::from("not set"),
    }
}

// Main Actix Web service
#[actix_web::main]
async fn actix_main(args: Args) -> std::io::Result<()> {
    // Initialize tracing logging using the args.<debug|verbose|...> specified
    // Fallback to using environmental variable RUST_LOG=<debug|info|...>
    let env_rust_log = get_env_var("RUST_LOG");
    let tracing_log_level = if args.debug || env_rust_log == *"debug" {
        Level::DEBUG
    } else if args.verbose || env_rust_log == *"info" {
        Level::INFO
    } else {
        Level::WARN
    };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing_log_level) // really the minimum log level
        //.with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .with_writer(std::io::stderr)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting the global default subscriber failed!");

    // Bring information from `args` into scope
    let database_files = args.database_files;
    // TODO: Makes sure the path provided in database_files exists and is read and writable

    // Prometheus middleware
    let prometheus = PrometheusMetricsBuilder::new("actix_data_receiver")
        .endpoint("/metrics")
        .build()
        .unwrap();

    // Initialize the HTTP server with the application
    info!("Starting actix-data-receiver");
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(prometheus.clone())
            .app_data(web::Data::new(AppData {
                database_files: database_files.clone(),
            }))
            .service(create_data)
            .service(ping)
    })
    .bind((args.addr, args.port))?
    .run()
    .await
}

// Configure command-line options
#[derive(Parser, Debug)]
#[command(
    about = "A simple data receiver which will save JSON formatted data into a SQLite database for later use.",
    long_about = None,
    version = None,
)]
struct Args {
    /// The IP address to listen for requests
    #[arg(short, long, default_value = "0.0.0.0")]
    addr: String,

    /// The port number to listen for requests
    #[arg(short, long, default_value_t = 8888)]
    port: u16,

    /// File path to where databases are located
    #[arg(long, default_value = "./")]
    database_files: String,

    /// Increase log messaging to verbose
    #[arg(short, long)]
    verbose: bool,

    /// Increase log messaging to debug
    #[arg(long)]
    debug: bool,
}

// CLI configuration options using clap
fn main() {
    let args = Args::parse();

    // TODO: Future support for standard in without a web frontend

    // Start the web service
    let _ = actix_main(args);
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::http::StatusCode;
    use actix_web::test;

    #[actix_web::test]
    async fn test_create_data() {
        // Initialize the application
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    database_files: String::from("./"),
                }))
                .service(create_data),
        )
        .await;

        // Send a request to the `client_address` endpoint
        // curl -i -X PUT -d '{"curl test": true}' http://localhost:8888/test/test
        //let timestamp: DateTime<Utc> = Utc::now();
        //let data = format!("{{'actix test': true, 'timestamp': {timestamp}}}");
        let data = "{'actix test': true, 'timestamp': 'timestamp'}";
        let req = test::TestRequest::put()
            .uri("/test/test")
            .set_payload(data.as_bytes())
            .to_request();

        // Send the request and parse the response as JSON
        let response = test::call_service(&app, req).await;

        // Assert the response is a 201 Created
        assert_eq!(response.status(), StatusCode::CREATED);

        // Post test, remove any database files created
    }

    #[actix_web::test]
    async fn test_ping() {
        // Initialize the application
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppData {
                    database_files: String::from("./"),
                }))
                .service(ping),
        )
        .await;

        // Send a request to the `client_address` endpoint
        let req = test::TestRequest::get().uri("/ping").to_request();

        // Send the request and parse the response as JSON
        let result: PongResponse = test::call_and_read_body_json(&app, req).await;

        // Assert the response
        assert_eq!(result.ping, String::from("pong"));
    }
}
