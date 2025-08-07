use warp::Filter;
use log::{info, warn};

#[tokio::main]
async fn main() {
    // Initialize the logger with info as default, but allow RUST_LOG to override
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    // Add logging middleware for requests
    let routes = hello.with(warp::log("api"));

    let port = 3030;
    let addr = ([0, 0, 0, 0], port);

    info!("🚀 Server starting...");
    info!("📡 Listening on http://0.0.0.0:{}", port);
    info!("🔗 Try: http://localhost:{}/hello/world", port);
    warn!("Press Ctrl+C to stop the server");

    warp::serve(routes)
        .run(addr)
        .await;
}
