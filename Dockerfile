FROM rust:1.84 AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y nodejs npm
COPY package.json package-lock.json ./
RUN npm install
COPY Cargo.toml Cargo.lock postcss.config.mjs tailwind.config.cjs webpack.config.cjs ./
COPY src/ src/
COPY assets/ assets/
COPY templates/ templates/
COPY migrations/ migrations/

RUN npm run build
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update
RUN apt-get install -y libssl3 ca-certificates nginx
RUN rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/name-lookup /usr/local/bin/name-lookup
COPY --from=builder /app/static /app/static
COPY ssl /etc/nginx/certs
COPY nginx.conf /etc/nginx/nginx.conf
COPY entrypoint.sh entrypoint.sh
RUN chmod +x entrypoint.sh

EXPOSE 80 443

CMD ["./entrypoint.sh"]
