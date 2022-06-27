use actix_web::{get, web, App, HttpResponse, HttpServer};
use chrono::{Datelike, TimeZone};
use dotenv::dotenv;
use listenfd::ListenFd;
use serde::{Deserialize, Serialize};
use std::env;

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
    let mut server = HttpServer::new(|| App::new().configure(employees::init_routes));
    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = env::var("HOST").expect("Please set host in .env");
            let port = env::var("PORT").expect("Please set port in .env");
            server.bind(format!("{}:{}", host, port))?
        }
    };
    server.run().await
}

#[get("/progress")]
async fn get_progress() -> Result<HttpResponse, CustomError> {
    let (line, percent) = web::block(|| progress()).await.unwrap();
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
    let today = chrono::offset::Local::now().date();
    let year_dt = chrono::Local.ymd(today.year(), 1, 1);
    let days = today.signed_duration_since(year_dt).num_days();
    let percent = (days * 100) / YEAR_DAYS;
    let position = (len * percent as i32) / 100;
    return (position, percent);
}
