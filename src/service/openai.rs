use std::convert::Infallible;

use axum::Json;
use axum::response::Sse;
use axum::response::sse::Event;
use axum::response::sse::KeepAlive;
use futures::Stream;
use futures::StreamExt;
use lazy_static::lazy_static;
use reqwest::Client;
use reqwest::Method;
use reqwest::Request;
use reqwest::RequestBuilder;
use reqwest::Url;
use reqwest::header;
use reqwest_eventsource::EventSource;

use crate::service::auth::Claims;

use super::idl::ChatCompletionRequest;
use super::idl::GptMessage;

lazy_static!(
    static ref API_BASE_URL: String = std::env::var("API_BASE_URL").expect("OPENAI_API_KEY must be setted");
    static ref OPENAI_API_KEY: String = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be setted");
);

#[axum_macros::debug_handler]
pub async fn chat_completion(
    user: Claims,
    Json(prompt): Json<GptMessage>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    tracing::info!("[openai_service] user [{}] request to the API, and request content: {:?}", user.username, &prompt);

    // let prompt = GptMessage { role: "a".to_string(), content: "b".to_string() };
    let stream = ChatCompletionStream::new(prompt);
    Sse::new(stream).keep_alive(KeepAlive::default())
}

struct ChatCompletionStream {
    event_source: EventSource,
}

impl Stream for ChatCompletionStream {
    type Item = Result<Event, Infallible>;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        tracing::debug!("[openai_service] poll_next");
        loop {
            let next = self.event_source.poll_next_unpin(cx)
                .map_err(|e| tracing::warn!("[openai_service] Warn: {}", e));
            match futures::ready!(next) {
                // On event opened
                Some(Ok(reqwest_eventsource::Event::Open)) => {
                    tracing::debug!("[openai_service] Chat Completions opened!");
                },
                // On received part of the data
                Some(Ok(reqwest_eventsource::Event::Message(message))) => {
                    tracing::debug!("[openai_service] Chat Completions received data: {:#?}", message);
                    // Convert reqwest_eventsource::Event to axum::response::sse::Event
                    let event = Event::default()
                        .data(message.data)
                        .id(message.id)
                        .event(message.event);
                    return std::task::Poll::Ready(Some(Ok(event)));
                },
                // Occured some unexpected error
                Some(Err(err)) => {
                    // logging the meaningful error
                    if type_of(&err) != "()" {
                        tracing::warn!("[openai_service] Error: {:?}", err);
                    }
                    self.event_source.close();
                    return std::task::Poll::Ready(None);
                },
                None => return std::task::Poll::Ready(None),
            }
        }
    }
}

fn type_of<T>(_: &T) -> &str {
    std::any::type_name::<T>()
}

impl ChatCompletionStream {
    /// Create a Chat Completion Stream
    fn new(prompt: GptMessage) -> Self {
        let chat_completions_request = ChatCompletionRequest::new("gpt-3.5-turbo", vec![prompt]);
        let body_json = serde_json::to_string(&chat_completions_request).unwrap();
        let url = Url::parse(&format!("{}/v1/chat/completions", API_BASE_URL.as_str()))
            .expect("[openai_service] Chat Completions Url is incorrect.");
        let request = Request::new(Method::POST, url);

        tracing::debug!("[openai_service] chat completion reqeust body: {}", body_json);

        let builder = RequestBuilder::from_parts(
            Client::new(), 
            request
            )
            .bearer_auth(OPENAI_API_KEY.to_string())
            .header(header::CONTENT_TYPE, "application/json;charset=utf-8")
            .body(body_json);

        let event_source = EventSource::new(builder)
            .expect("[openai_service] EventSource Create Error");
        
        ChatCompletionStream {
            event_source
        }
    }
}
