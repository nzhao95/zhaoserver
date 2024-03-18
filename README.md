# Zhaoserver
A web server implemented in Rust cause we are cool kids
This package should give you a minimalistic setup of a **Rust** web backend running with a **PostgreSQL** database using **Docker**. 

# Setup 

Install **Docker** or make sure the Docker daemon is running. Open a command prompt at the project location and build the docker image for the server : 


```
docker build -t zserver zserver/.
```

Launch the server with Docker Compose

```
docker compose up 
```

You should be able to access your server on port 8888:8888. Open you web browser and go to http://localhost:8888

# Weather query exemple

Currently in the build there is a weather query backend with a PostgreSQL database that runs weather queries on the open-meteo api. This was done following the instructions by shuttle in this webpost : https://www.shuttle.rs/blog/2023/09/27/rust-vs-go-comparison
