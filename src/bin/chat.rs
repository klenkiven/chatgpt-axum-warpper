use std::time::Duration;

use futures::StreamExt;
use reqwest::{Url, Method, RequestBuilder, Client, header, Request};
use reqwest_eventsource::{EventSource, Event};

use lazy_static::lazy_static;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

lazy_static!(
    static ref API_BASE_URL: String = std::env::var("API_BASE_URL").expect("OPENAI_API_KEY must be setted");
    static ref OPENAI_API_KEY: String = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be setted");
);

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let subscriber = FmtSubscriber::builder()
        // Set log level for subscriber
        .with_max_level(Level::TRACE)
        // Use a more compact, abbreviated log format
        .compact()
        // Display source code file paths and code line numbsers
        // .with_file(true)
        // .with_line_number(true)
        // Display the thread ID an event was recorded on
        .with_thread_ids(true)
        // Don't display the event's target (module path)
        .with_target(false)
        // Build the subscriber
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

        let body_json = 
        r#"{
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "user",
                    "content": "Please write a qiuck sort"
                }
            ],as_str()
            "stream": true,
            "temperature": 0.5
        }"#;
        let url = Url::parse(&format!("{}/v1/chat/completions", API_BASE_URL.as_str()))
            .expect("[openai_service] Chat Completions Url is incorrect.");
        let request = Request::new(Method::POST, url);

        tracing::debug!("[openai_service] chat completion reqeust body: {}", body_json);

        let builder = RequestBuilder::from_parts(
            Client::new(), 
            request
            )
            .bearer_auth(OPENAI_API_KEY.to_string())
            .header(header::CONTENT_TYPE, "application/json")
            .body(body_json)
            .timeout(Duration::from_secs(24 * 60 * 60));
        
        let mut es = EventSource::new(builder)
            .expect("[openai_service] EventSource Create Error");
        
    while let Some(event) = es.next().await {
        match event {
            Ok(Event::Open) => println!("Connection Open!"),
            Ok(Event::Message(message)) => println!("Message: {:#?}", message),
            Err(err) => {
                println!("Error: {}", err);
                es.close();
            }
        }
    }
}