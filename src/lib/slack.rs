// use futures::executor::block_on;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use serde_json::json;
use slog::{error, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct SlackClient {}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlackMessage {
    pub subject: String,
    pub state: String,
    pub id: String,
    pub resource: String,
    pub git_repo: String,
    pub git_tag: String,
    pub git_sha: String,
}

#[derive(Debug)]
pub struct SlackMessageSend {}

pub struct SlackThread {
    channel: crossbeam::Receiver<String>,
    logger: slog::Logger,
}

impl SlackClient {

    pub async fn call(message: &SlackMessage, logger: &slog::Logger) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let api_token = dotenv::var("SLACK_API_TOKEN").unwrap();
        let channel_name = dotenv::var("SLACK_CHANNEL_NAME").unwrap();
        let username = dotenv::var("SLACK_USERNAME").unwrap();

        let color = match message.state.as_str() {
            "error" => {
                dotenv::var("SLACK_COLOR_ERROR").unwrap()
            },
            "pending" => {
                dotenv::var("SLACK_COLOR_PENDING").unwrap()
            },
            "success" => {
                dotenv::var("SLACK_COLOR_SUCCESS").unwrap()
            },
            _ => {
                dotenv::var("SLACK_COLOR_PENDING").unwrap()
            }
        };

        let https = HttpsConnector::new();
        let client = Client::builder()
            .build::<_, hyper::Body>(https);

        let pretext = format!("*{} : {}*", username, message.id);

        let text_lines: [String; 4] = [
            format!("resource: {}", message.resource),
            format!("git_repo: {}", message.git_repo),
            format!("git_tag: {}", message.git_tag),
            format!("git_sha: {}", message.git_sha),
        ];

        let attachment = json!({
            "color": color,
            "pretext": pretext,
            "text": text_lines.join("\n"),  // newline separated lines of text
            "title": message.subject,
        });

        let msg = json!({
            "as_user": "false",
            "attachments": [attachment],
            "channel": channel_name,
            "username": username,
        });

        // info!(logger, "slack_message: {}", msg.to_string());

        let req = Request::builder()
            .method(Method::POST)
            .uri("https://slack.com/api/chat.postMessage")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", api_token))
            .body(Body::from(msg.to_string()))?;

        match client.request(req).await {
            Err(e) => {
                error!(logger, "slack_thread_chat_postmessage_exception: {}", e);
            },
            Ok(_) => {
            }
        };

        Ok(())
    }
}

impl SlackMessageSend {
    pub fn call(sender: &crossbeam::Sender<String>, message: &SlackMessage) -> Result<(), Box<dyn std::error::Error>> {
        let j = serde_json::to_string(&message).unwrap();

        sender.send(j).unwrap();

        Ok(())
    }
}

impl SlackThread {

    pub fn new(channel: crossbeam::Receiver<String>, logger: slog::Logger) -> SlackThread {
        SlackThread {
            channel: channel,
            logger: logger,
        }
    }

    pub async fn call(&self) {
        info!(self.logger, "slack_publisher_starting");

        loop {
            let data = self.channel.recv().unwrap();

            // info!(self.logger, "slack_thread_message"; "data" => &data);

            // todo: try chaining here

            // parse the json string into a SlackMessage object
            match serde_json::from_str(&data) {
                Err(e) => {
                    // whoops

                    error!(self.logger, "slack_thread_exception: {}", e);
                },
                Ok(object) => {
                    let message: SlackMessage = object;

                    // call future and block with await
                    match SlackClient::call(&message, &self.logger).await {
                        Ok(_) => {},
                        Err(e) => {
                            error!(self.logger, "slack_thread_exception: {}", e);
                        }
                    }

                    ()
                }
            };
        }
    }

}
