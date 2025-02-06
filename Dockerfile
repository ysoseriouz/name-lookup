FROM rust:1.84

WORKDIR /usr/src/name-lookup

RUN apt-get update && apt-get install -y nodejs npm
COPY package.json package-lock.json ./
RUN npm install
COPY webpack.config.cjs tailwind.config.cjs postcss.config.mjs ./
COPY assets ./assets
RUN npm run build
COPY . .
RUN cargo install --path .

EXPOSE 3000
CMD ["name-lookup"]
