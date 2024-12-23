use tracing::{level_filters::LevelFilter, subscriber::DefaultGuard, Level};
use tracing_subscriber::Layer;
use tracy_client::Client;
use {tracing_subscriber::layer::SubscriberExt, tracing_subscriber::util::SubscriberInitExt};

pub struct Tracing {
    _default_guard: DefaultGuard,
}

impl Tracing {
    pub fn setup() -> Self {
        let tracy_layer = tracing_tracy::TracyLayer::default();
        let fmt = tracing_subscriber::fmt::layer().with_filter(LevelFilter::TRACE);
        let default_guard = tracing_subscriber::registry()
            .with(fmt)
            .with(tracy_layer)
            .set_default();
        Self {
            _default_guard: default_guard,
        }
    }
}
