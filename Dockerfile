FROM rust:latest as builder

RUN rustup component add rustfmt

WORKDIR /work

COPY . /work/

RUN cargo build

# ----------

FROM debian:latest

COPY --from=builder /work/target/debug/one /one
COPY --from=builder /work/target/debug/two /two
COPY --from=builder /work/target/debug/three /three

EXPOSE 3000
EXPOSE 4000
EXPOSE 5000
