FROM docker.io/rust:1.61.0 as builder

WORKDIR /usr/src/track-wear
COPY . .
RUN cargo install --path .

FROM docker.io/debian:buster-slim

RUN apt-get update && apt-get install -y libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/track-wear-backend /usr/local/bin/track-wear-backend

ENV GOOGLE_CLIENT_ID=
ENV IMAGE_FOLDER=/images
ENV ROCKET_SECRET_KEY=
ENV ROCKET_DATABASES=

CMD ["track-wear-backend"]
