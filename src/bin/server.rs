use std::net::SocketAddr;

use axum::{
    routing::{ get, post }, 
    Router
};

use gpt_axum_test::service::{user, openai};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // Init Tracing Log
    let subscriber = FmtSubscriber::builder()
        // Set log level for subscriber
        .with_max_level(Level::TRACE)
        // Use a more compact, abbreviated log format
        .compact()
        // Display source code file paths and code line numbsers
        .with_file(false)
        .with_line_number(false)
        // Display the thread ID an event was recorded on
        .with_thread_ids(true)
        // Don't display the event's target (module path)
        .with_target(true)
        // Build the subscriber
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/login", post(user::login))
        .route("/chat", post(openai::chat_completion))
        .nest(
            "/user", 
            Router::new()
                .route("/register", post(user::register))
        );
    
    let addr = "[::]:3000".parse::<SocketAddr>().unwrap();
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}