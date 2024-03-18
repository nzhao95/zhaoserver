# Zhaoserver
A web server implemented in Rust cause we are cool kids
This package should give you a minimalistic setup of a **Rust** web backend running with a **PostgreSQL** database using **Docker**. 

# Setup 

Install **Docker** then run the open a command prompt at the project location and run the following commands 

```
docker build -t zhaoserver .
docker compose up 
```

# Weather query exemple

Currently in the build there is a weather query backend with a PostgreSQL database that runs weather queries on the open-meteo api. This was done following the instructions by shuttle in this webpost : https://www.shuttle.rs/blog/2023/09/27/rust-vs-go-comparison
