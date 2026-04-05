FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY target/release/backend /app/backend
COPY backend/Rocket.toml /app/Rocket.toml

EXPOSE 8000

CMD ["./backend"]
