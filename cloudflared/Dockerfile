FROM cloudflare/cloudflared:latest

ENV TUNNEL_ORIGIN_CERT=/etc/cloudflared/cert.pem
ENV TUNNEL_TRANSPORT_PROTOCOL=http2
ENV NO_AUTOUPDATE=false

WORKDIR /etc/cloudflared

COPY --chown=nonroot cloudflared/cert.pem cert.pem
COPY --chown=nonroot cloudflared/credentials.json credentials.json
COPY --chown=nonroot cloudflared/config.yml config.yml

CMD ["tunnel", "--config", "config.yml", "run"]
