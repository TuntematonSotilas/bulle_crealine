# Get started with a build env with Rust nightly
FROM rust:1.95.0-alpine3.22 AS builder

RUN apk update && \
    apk add --no-cache bash curl npm libc-dev binaryen perl make

RUN npm install -g sass
# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /work
COPY . .

RUN cargo install --locked cargo-leptos

RUN npm i

RUN cargo leptos build --release

FROM rust:1.95.0-alpine3.22 AS runner

WORKDIR /app

COPY --from=builder /work/target/release/bulle_crealine /app/
COPY --from=builder /work/target/site /app/site
COPY --from=builder /work/Cargo.toml /app/

# Stamp the bundle with a digest of its own contents, so that every build which
# changes it changes its URL -- and every build which does not keeps it, leaving
# the caches of returning visitors warm. The server reads the name back off the
# disk, so nothing here has to agree with an environment variable.
RUN set -eux; \
    cd /app/site/pkg; \
    stamp="$(cat bulle_crealine.js bulle_crealine.wasm bulle_crealine.css | md5sum | cut -c1-12)"; \
    for ext in js wasm css; do mv "bulle_crealine.$ext" "bulle_crealine.$stamp.$ext"; done; \
    ls -l

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT=./site
EXPOSE 8080

CMD ["/app/bulle_crealine"]