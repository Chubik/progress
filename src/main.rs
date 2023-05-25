use actix_web::{get, web, App, HttpResponse, HttpServer};
use chrono::{Datelike, TimeZone};
use dotenv::dotenv;
use listenfd::ListenFd;
use serde::{Deserialize, Serialize};
use std::env;

mod error_handlers;
use error_handlers::CustomError;

const BACK_SYMBOL: &str = "▒";
const SELECT_SYMBOL: &str = "▓";
const YEAR_DAYS: i64 = 365;

#[derive(Serialize, Deserialize)]
struct Response {
    progress: String,
    percent: i64,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(|| App::new().configure(init_routes));
    let (host, port) = match listenfd.take_tcp_listener(0)? {
        Some(listener) => {
            let addr = listener.local_addr().unwrap();
            server = server.listen(listener)?;
            (addr.ip().to_string(), addr.port().to_string())
        }
        None => {
            let host = env::var("HOST").expect("Please set host in .env");
            let port = env::var("PORT").expect("Please set port in .env");
            server = server.bind(format!("{}:{}", host, port))?;
            (host, port)
        }
    };
    println!("Server running on {}:{}", host, port);
    server.run().await
}

fn init_routes(config: &mut web::ServiceConfig) {
    config.service(get_progress);
}

#[get("/progress")]
async fn get_progress() -> Result<HttpResponse, CustomError> {
    let (line, percent) = web::block(|| progress())
        .await
        .map_err(|_| CustomError::new(500, "Internal Server Error".to_string()))?;

    let response = Response {
        progress: line,
        percent: percent,
    };
    let serialized_response = serde_json::to_string(&response).unwrap();
    Ok(HttpResponse::Ok().json(serialized_response))
}

fn progress() -> (String, i64) {
    let line_len = 20;
    let (till, percent) = count_percent(line_len);
    let mut range_str = String::from("");

    for n in 0..line_len {
        if n > till {
            range_str.push_str(BACK_SYMBOL);
        } else {
            range_str.push_str(SELECT_SYMBOL);
        }
    }

    return (String::from(range_str), percent);
}

fn count_percent(len: i32) -> (i32, i64) {
    let (today, year_dt) = get_today_and_start_of_year();
    let days = today.signed_duration_since(year_dt).num_days();
    let percent = (days * 100) / YEAR_DAYS;
    let position = (len * percent as i32) / 100;
    return (position, percent);
}

fn get_today_and_start_of_year() -> (chrono::Date<chrono::Local>, chrono::Date<chrono::Local>) {
    let today = chrono::offset::Local::now().date();
    let year_dt = chrono::Local.ymd(today.year(), 1, 1);
    return (today, year_dt);
}
