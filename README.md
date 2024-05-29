# tracing_playground

`tracing_playground` is sample implementation of applications written in Rust and trace using jaeger.
The communication between applications is http with Axum and http2 with tonic.

# how to run

## run jaeger

`$ docker run -d -p6831:6831/udp -p6832:6832/udp -p16686:16686 -p14268:14268 jaegertracing/all-in-one:latest`

## run applications

```sh
RUST_LOG=info TWO_URL=http://localhost:4000/ cargo run -p one
RUST_LOG=info THREE_URL=http://localhost:5000/ cargo run -p two
RUST_LOG=info cargo run -p three
```

## request

Exec `curl localhost:3000` and open `localhost:16686` in your browser.

# images

![image](https://user-images.githubusercontent.com/21954399/155303798-bd11d8b7-e62a-4749-85df-d4394a970fd4.png)

![image](https://user-images.githubusercontent.com/21954399/155303958-3c543dbc-7f62-4e34-9100-a693dad8c1ba.png)

![image](https://user-images.githubusercontent.com/21954399/155304002-d885be05-09f2-4dc1-9552-b9354e096fa6.png)
