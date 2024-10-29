use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use slog::{error, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct SlackChatAttachment {
    pub color: String,
    pub pretext: String,
    pub text: String,
    pub title: String,
}

#[derive(Debug)]
pub struct SlackChatPost {}

#[derive(Debug)]
pub struct SlackChatPublish {}

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
pub struct SlackThread {
    channel: crossbeam_channel::Receiver<String>,
    logger: slog::Logger,
}

impl SlackChatPost {
    pub fn call(message: &SlackMessage) -> Result<(), Box<dyn std::error::Error>> {
        let api_token = dotenv::var("SLACK_API_TOKEN").unwrap();
        let channel_name = dotenv::var("SLACK_CHANNEL_NAME").unwrap();
        let username = dotenv::var("SLACK_USERNAME").unwrap();

        let pretext = format!("*{} : {}*", username, message.id);
        let title = message.subject.to_string();

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

        let text_vec: [String; 4] = [
            format!("resource: {}", message.resource),
            format!("git_repo: {}", message.git_repo),
            format!("git_tag: {}", message.git_tag),
            format!("git_sha: {}", message.git_sha),
        ];

        let text_lines = text_vec.join("\n");

        let attachments = vec![
            SlackChatAttachment {
                color: color,
                pretext: pretext,
                text: text_lines,
                title: title,
            }
        ];

        let slack_chat_json = json!({
            "as_user": "false",
            "attachments": attachments,
            "channel": channel_name,
            "username": username,
        }).to_string();

        let client = reqwest::blocking::Client::new();

        let _result = client.post("https://slack.com/api/chat.postMessage")
            .header(AUTHORIZATION, format!("Bearer {}", api_token))
            .header(CONTENT_TYPE, "application/json")
            .body(slack_chat_json)
            .send()?;

        // println!("result: {:?}", result);

        Ok(())
    }

}

impl SlackChatPublish {
    pub fn call(sender: &crossbeam_channel::Sender<String>, message: &SlackMessage) -> Result<(), Box<dyn std::error::Error>> {
        let j = serde_json::to_string(&message).unwrap();

        sender.send(j).unwrap();

        Ok(())
    }
}

impl SlackThread {

    pub fn new(channel: crossbeam_channel::Receiver<String>, logger: slog::Logger) -> SlackThread {
        SlackThread {
            channel: channel,
            logger: logger,
        }
    }

    pub fn call(&self) {
        info!(self.logger, "slack_thread_starting");

        loop {
            let data = match self.channel.recv() {
                Err(e) => {
                    error!(self.logger, "slack_thread_exception: {}", e);

                    return ()
                },
                Ok(data) => {
                    data
                }
            };

            // parse the json string into a SlackMessage object
            let message: SlackMessage = match serde_json::from_str(&data) {
                Err(e) => {
                    error!(self.logger, "slack_thread_exception: {}", e);

                    continue
                },
                Ok(object) => {
                    object
                }
            };

            match SlackChatPost::call(&message) {
                Ok(_) => {},
                Err(e) => {
                    error!(self.logger, "slack_thread_exception: {}", e);
                }
            }

            ()
        }
    }

}
