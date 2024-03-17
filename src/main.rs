use axum::{http::StatusCode, routing::get, Router};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use askama::Template;
use dotenv::dotenv;
use serde::Deserialize;
use sqlx::PgPool;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use anyhow::Context;

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
	fn into_response(self) -> Response {
    	(
        	StatusCode::INTERNAL_SERVER_ERROR,
        	format!("Something went wrong: {}", self.0),
    	)
        	.into_response()
	}
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
	E: Into<anyhow::Error>,
{
	fn from(err: E) -> Self {
    	Self(err.into())
	}
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

async fn index() -> IndexTemplate {
	IndexTemplate
}

#[derive(Deserialize)]
pub struct WeatherQuery {
	pub city: String,
}

async fn weather(Query(weather_query) : Query<WeatherQuery>, State(pool) : State<PgPool>) -> Result<WeatherDisplay, AppError> {
	
	let city = &weather_query.city;
    let lat_long = get_lat_long(&pool, city)
        .await?;
	let weather = fetch_weather(&lat_long)
    	.await?;
    
	Ok(WeatherDisplay::new(&city, &weather))
}

async fn stats() -> &'static str {
	"Stats"
}

#[derive(Deserialize)]
pub struct GeoResponse {
	pub results: Vec<LatLong>,
}


#[derive(sqlx::FromRow, Deserialize, Debug, Clone)]
pub struct LatLong {
	pub latitude: f64,
	pub longitude: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WeatherResponse {
	pub latitude: f64,
	pub longitude: f64,
    pub timezone : String,
    pub hourly : Hourly
}

#[derive(Deserialize, Debug, Clone)]
pub struct Hourly {
    pub time : Vec<String>,
    pub temperature_2m : Vec<f64>
}

#[derive(Template, Deserialize, Debug, Clone)]
#[template(path = "weather.html")]
pub struct WeatherDisplay {
    pub city : String, 
    pub forecasts : Vec<Forecast>
}

#[derive(Deserialize, Debug, Clone)]
pub struct Forecast {
    pub date : String, 
    pub temperature : String
}

impl WeatherDisplay {
    pub fn new(city : &str, response : &WeatherResponse) -> WeatherDisplay {
        WeatherDisplay {
            city : city.to_owned(),
            forecasts : response.hourly.time
            .iter()
            .zip(response.hourly.temperature_2m.iter())
            .map(|(time, temperature)| Forecast { date : time.clone(),
                temperature : temperature.to_string()})
            .collect()
        }
    }
}

async fn get_lat_long(pool :&PgPool, city_name: &str) -> Result<LatLong, anyhow::Error> {
    let lat_long = sqlx::query_as::<_, LatLong>(
        "SELECT lat AS latitude, long AS longitude FROM cities WHERE name = $1",
    )
    .bind(city_name)
    .fetch_optional(pool)
    .await?;

    if let Some(lat_long) = lat_long {
        return Ok(lat_long);
    }

    let lat_long = fetch_lat_long(city_name).await?;

    sqlx::query("INSERT INTO cities (name, lat, long) VALUES ($1, $2, $3)" )
    .bind(city_name)
    .bind(lat_long.latitude)
    .bind(lat_long.longitude)
    .execute(pool)
    .await?;

    Ok(lat_long)
} 

async fn fetch_lat_long(city : &str) -> Result<LatLong, anyhow::Error> {
    let endpoint = format!(
    	"https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
    	city
	);

    let response = reqwest::get(&endpoint)
    .await?
    .json::<GeoResponse>()
    .await?;
    response.results
    .get(0)
    .cloned()
    .context("")
    
}

async fn fetch_weather(lat_long : &LatLong) -> Result<WeatherResponse, anyhow::Error> {
    let endpoint = format!(
    	"https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m",
    	lat_long.latitude, lat_long.longitude
	);

    let response = reqwest::get(&endpoint)
    .await?.json::<WeatherResponse>().await?;

    Ok(response)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting Zhao Server");
    dotenv().ok();
	let db_connection_str = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
	let pool = sqlx::PgPool::connect(&db_connection_str)
    	.await
    	.context("can't connect to database")?;

	let app = Router::new()
    	.route("/", get(index))
    	.route("/weather", get(weather))
    	.route("/stats", get(stats))
    	.with_state(pool);

	let addr = SocketAddr::from(([127, 0, 0, 1], 8888));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, app.into_make_service()).await?;
    
	Ok(())
}