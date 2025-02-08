FROM rust:1.84 AS builder

WORKDIR /usr/src/name-lookup

RUN apt-get update && apt-get install -y nodejs npm
COPY package.json package-lock.json ./
RUN npm install
COPY . .
RUN npm run build
RUN cargo install --path .

FROM debian:bookworm-slim

RUN apt-get update
RUN apt-get install -y libssl3 ca-certificates
RUN rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/local/cargo/bin/name-lookup /usr/local/bin/name-lookup
COPY --from=builder /usr/src/name-lookup/static /app/static

EXPOSE 3000
CMD ["name-lookup"]
