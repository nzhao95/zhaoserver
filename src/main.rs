use axum::{extract::Query, http::{response, StatusCode}, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{collections::HashMap, net::SocketAddr};
use tokio::net::TcpListener;

// basic handler that responds with a static string
async fn index() -> &'static str {
	"Index"
}

#[derive(Deserialize)]
pub struct WeatherQuery {
	pub city: String,
}

async fn weather(Query(weather_query) : Query<WeatherQuery>) -> Result<String, StatusCode> {
	
	let city = &weather_query.city;
    let lat_long = fetch_lat_long(city)
        .await
    	.map_err(|_| StatusCode::NOT_FOUND)?;
	let weather = fetch_weather(&lat_long)
    	.await
    	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
	Ok(format!("The weather in {} is {}°C", city, weather.hourly.temperature_2m[0]))
}

async fn stats() -> &'static str {
	"Stats"
}

#[derive(Deserialize)]
pub struct GeoResponse {
	pub results: Vec<LatLong>,
}


#[derive(Deserialize, Debug, Clone)]
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

async fn fetch_lat_long(city : &str) -> Result<LatLong, Box<dyn std::error::Error>> {
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
    .ok_or("No results found".into())
}

async fn fetch_weather(lat_long : &LatLong) -> Result<WeatherResponse, Box<dyn std::error::Error>> {
    let endpoint = format!(
    	"https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m",
    	lat_long.latitude, lat_long.longitude
	);

    let response = reqwest::get(&endpoint)
    .await?.json::<WeatherResponse>().await?;

    Ok(response)
}

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/", get(index))
        .route("/weather", get(weather))
        .route("/stats", get(stats));

    let addr = SocketAddr::from(([127,0,0,1], 8888));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, router)
        .await
        .unwrap();
}