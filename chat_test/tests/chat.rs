use std::net::SocketAddr;

use chat_core::{Chat, ChatType, Message};
use futures::StreamExt;
use reqwest::StatusCode;
use reqwest_eventsource::{Event, EventSource};
use serde::Serialize;
use serde_json::json;
use tokio::net::TcpListener;
use tracing::info;

#[derive(Debug, Serialize)]
pub struct AuthToken {
    pub token: String,
}

pub struct ChatServer {
    pub addr: SocketAddr,
    pub token: String,
    pub client: reqwest::Client,
}

struct NotifyServer;

const WILD_ADDR: &str = "0.0.0.0:0";

#[tokio::test]
async fn chat_server_should_work() -> anyhow::Result<()> {
    let (tdb, state) = chat_server::AppState::new_for_test().await?;

    let server = ChatServer::new(&mut state.clone()).await?;

    let db_url = tdb.url();
    dbg!(format!("token_0:{}", server.token));
    let token = server.token.rsplit(" ").next().unwrap();
    dbg!(format!("token: {}", token));

    let _notify_server = NotifyServer::new(&db_url, token).await?;

    server.create_chat().await?;
    server.create_message().await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    Ok(())
}

impl NotifyServer {
    pub async fn new(db_url: &str, token: &str) -> anyhow::Result<Self> {
        let mut config = notify_server::AppConfig::load()?;
        config.server.db_url = db_url.to_string();

        let app = notify_server::get_router(config).await?;

        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        let mut es = EventSource::get(format!("http://{}/events?token={}", addr, token));

        tokio::spawn(async move {
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => println!("connection opened"),
                    Ok(Event::Message(message)) => match message.event.as_str() {
                        "new_chat" => {
                            let chat: chat_core::Chat =
                                serde_json::from_str(message.data.as_str()).unwrap();
                            assert_eq!(chat.id, 9);
                            assert_eq!(chat.name, Some("public chat".to_string()));
                            assert_eq!(chat.members, vec![1, 2]);
                        }
                        "new_message" => {
                            let message: chat_core::Message =
                                serde_json::from_str(message.data.as_str()).unwrap();
                            assert_eq!(message.id, 13);
                            assert_eq!(message.content, Some("hello".to_string()));
                            assert_eq!(message.files, Some(vec![]));
                        }
                        _ => {
                            panic!("unexpected event:{:?}", message);
                        }
                    },
                    Err(e) => {
                        println!("error: {:?}", e);
                        es.close();
                    }
                }
            }
        });

        Ok(Self)
    }
}

impl ChatServer {
    pub async fn new(state: &mut chat_server::AppState) -> anyhow::Result<Self> {
        let app = chat_server::get_router(state).await?;

        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;
        dbg!("Listening on: {}", &addr);

        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
            info!("Server stopped");
        });

        let mut ret = Self {
            addr,
            token: "".to_string(),
            client: reqwest::Client::new(),
        };

        ret.token = ret.signin().await?;

        Ok(ret)
    }

    pub async fn signin(&self) -> anyhow::Result<String> {
        let resp = self
            .client
            .post(format!("http://{}/api/signin", self.addr))
            .header("Content-Type", "application/json")
            .json(&json!({
                "email": "test@yahoo.com",
                "password": "1234567"
            }))
            .send()
            .await?;

        assert_eq!(resp.status(), StatusCode::CREATED);

        let token = resp
            .headers()
            .get("authorization")
            .unwrap()
            .to_str()
            .unwrap();

        Ok(token.to_string())
    }

    pub async fn create_chat(&self) -> anyhow::Result<()> {
        let resp = self
            .client
            .post(format!("http://{}/api/chats", self.addr))
            .header("Content-Type", "application/json")
            .header("Authorization", &self.token)
            .json(&json!({
                "name": "public chat",
                "members": [1,2],
                "public": true
            }))
            .send()
            .await?;
        let status = resp.status();
        let chat: Chat = resp.json().await?;
        assert_eq!(chat.id, 9);
        assert_eq!(chat.name, Some("public chat".to_string()));
        assert_eq!(chat.r#type, ChatType::PublicChannel);

        assert_eq!(status, StatusCode::CREATED);

        Ok(())
    }

    pub async fn create_message(&self) -> anyhow::Result<()> {
        let resp = self
            .client
            .post(format!("http://{}/api/chats/9", self.addr))
            .header("Content-Type", "application/json")
            .header("Authorization", &self.token)
            .json(&json!({
                "content": "hello",
                "files": [],
            }))
            .send()
            .await?;
        assert_eq!(resp.status(), 200);
        let msg: Message = resp.json().await?;
        assert_eq!(msg.chat_id, 9);
        assert_eq!(msg.content, Some("hello".to_string()));
        assert_eq!(msg.files, Some(vec![]));

        Ok(())
    }
}
