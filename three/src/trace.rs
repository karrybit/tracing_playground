fn tracing_interceptor<S>(service: S) -> InterceptedService<S> {
    InterceptedService::new(service)
}

#[derive(Clone, Debug)]
struct InterceptedService<S> {
    inner: S,
}

impl<S> InterceptedService<S> {
    pub fn new(service: S) -> Self {
        Self { inner: service }
    }
}

impl<S> tower::Service<hyper::Request<hyper::Body>> for InterceptedService<S>
where
    S: tower::Service<
            hyper::Request<hyper::Body>,
            Response = hyper::Response<tonic::body::BoxBody>,
        > + tonic::transport::NamedService
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: hyper::Request<hyper::Body>) -> Self::Future {
        let mut svc = self.inner.clone();
        tracing::info!("request in call: {:?}", request);
        tracing::info!("header in call: {:?}", request.headers());

        let parent_ctx = opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&opentelemetry_http::HeaderExtractor(&request.headers()))
        });
        tracing::info!("parent ctx in call: {:?}", parent_ctx);
        tracing::info!("parent baggage in call: {:?}", parent_ctx.baggage());
        tracing::info!(
            "span context in hello: {:?}",
            parent_ctx.span().span_context()
        );
        tracing::info!("parent span ctx in call: {:?}", parent_ctx.span());

        let span = tracing::info_span!("call");
        span.set_parent(parent_ctx);
        tracing::info!("context in call: {:?}", span.context());
        tracing::info!("baggage in call: {:?}", span.context().baggage());
        tracing::info!(
            "span context in call: {:?}",
            span.context().span().span_context()
        );
        tracing::info!("span in call: {:?}", span.context().span());

        Box::pin(async move { span.in_scope(|| svc.call(request)).await })
    }
}

impl<S: tonic::transport::NamedService> tonic::transport::NamedService for InterceptedService<S> {
    const NAME: &'static str = S::NAME;
}
