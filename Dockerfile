FROM rust:1.84 AS builder

WORKDIR /usr/src/name-lookup

RUN apt-get update && apt-get install -y nodejs npm
COPY package.json package-lock.json ./
RUN npm install
COPY Cargo.toml Cargo.lock postcss.config.mjs tailwind.config.cjs webpack.config.cjs ./
COPY src/ src/
COPY assets/ assets/
COPY templates/ templates/
COPY migrations/ migrations/

RUN npm run build
RUN cargo install --path .

FROM debian:bookworm-slim

RUN apt-get update
RUN apt-get install -y libssl3 ca-certificates nginx
RUN rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/local/cargo/bin/name-lookup /usr/local/bin/name-lookup
COPY --from=builder /usr/src/name-lookup/static /app/static
COPY ssl /etc/nginx/certs
COPY nginx.conf /etc/nginx/nginx.conf
COPY entrypoint.sh entrypoint.sh
RUN chmod +x entrypoint.sh

EXPOSE 443

CMD ["./entrypoint.sh"]
