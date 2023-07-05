use engine::{EngineConfig, Schema};
use futures::SinkExt;
use opentelemetry::{
    sdk::{trace, Resource},
    KeyValue,
};
use synapse_common::{Duration, Time};
use synapse_engine as engine;
use synapse_tensor as tensor;
use tensor::{Tensor, Tensor1, TensorType};
use tokio_stream::StreamExt;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "synapse",
            )])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(
            tracing_subscriber::fmt::layer()
                .with_filter(tracing_subscriber::EnvFilter::new("INFO")),
        )
        .init();

    let config = EngineConfig::new().with_serve_metrics("0.0.0.0:8888");
    let sy = engine::Engine::start_with_config("file:///tmp/synapse/", config).await?;

    let schema = Schema::builder()
        .field("time")
        .data_type(TensorType::Timestamp)
        .required(true)
        .index(true)
        .finish()
        .field("i")
        .data_type(TensorType::Int32)
        .finish()
        .field("dt")
        .data_type(TensorType::Duration)
        .finish()
        .field("x")
        .data_type(TensorType::Float32)
        .row_shape((512,))
        .finish()
        .field("y")
        .data_type(TensorType::String)
        .row_shape((2,))
        .finish()
        .build();

    let pb = sy.topic("point").get_or_create(schema).await?.publish();
    let mut sink = pb.rows(1)?;

    let start = synapse_common::now();
    let end = start + Duration::seconds(5);
    let mut i = 0_i32;
    while synapse_common::now() < end {
        i += 1;
        sink.feed((
            synapse_common::now(),
            i,
            Duration::milliseconds(50),
            Tensor::linspace(i as f32, (i + 1) as f32, 512),
            tensor::tensor!["A".to_string(), "B".to_string()],
        ))
        .await?;
    }
    sink.close().await?;
    drop(sink);

    let mut rows = sy
        .query("SELECT * FROM point ORDER BY time")
        .await?
        .rows::<(Time, i32, Duration, Tensor1<f32>, Tensor1<String>)>()
        .await?;
    while let Some(row) = rows.try_next().await? {
        println!("{:?}", row);
    }

    sy.shutdown().await?;
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}