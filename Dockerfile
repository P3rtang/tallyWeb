FROM ubuntu:jammy as runner

RUN mkdir -p /app

# Copy the server binary to the /app directory
COPY target/server/release/tally_web /app/

COPY target/front /app/front
# /target/site contains our JS/WASM/CSS, etc.
COPY target/site /app/site

COPY .env-docker .env

WORKDIR /app

# Set any required env variables and
ENV RUST_LOG="info"
ENV APP_ENVIRONMENT="production"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="site"
EXPOSE 3000
# Run the server
CMD ["/app/tally_web"]
