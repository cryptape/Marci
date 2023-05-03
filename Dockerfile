FROM rust:1.67.1 as build
WORKDIR /usr/src/marci
LABEL author="Code Monad<codemonad@cryptape.com>"
COPY . .
RUN cargo build --release

FROM ubuntu:20.04

WORKDIR /app/marci
COPY --from=build /usr/src/marci/target/release/marci /app/marci
COPY --from=build /usr/src/marci/dist /app/marci/dist
ENV DB_URL="postgres://postgres:postgres@postgres/ckb"
ENV BIND="0.0.0.0:1800"
CMD ["./marci --db-url $DB_URL --bind $BIND"]