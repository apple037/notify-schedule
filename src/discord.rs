use serenity::{
    async_trait,
    model::{channel::Message, id::ChannelId},
    prelude::*,
    builder::{CreateEmbed, CreateMessage},
    http::Http,
};
use serde_derive::Deserialize;

struct Handler;

// Structs
#[derive(Deserialize, Debug)]
struct Config {
    discord: Discord,
}

#[derive(Deserialize, Debug)]
struct Discord {
    api_key: String,
    channel_id: u64,
}


#[async_trait]
impl EventHandler for Handler {}
// Public methods
pub async fn send_message_to_channel(message: String) {
    let config = get_config_from_toml();
    let token = config.api_key;
    let channel_id = config.channel_id;
    let channel_id = ChannelId::new(channel_id);
    let http = Http::new(&token);
    let res = channel_id.say(http, message).await;
    match res {
        Ok(_) => {
            tracing::info!("Message sent");
            println!("Message sent");
        }
        Err(e) => {
            tracing::error!("Message failed to send: {}", e);
            println!("Message failed to send: {}", e);
        }
    
    }
}

// Private methods
fn get_config_from_toml() -> Discord {
    let config_value: String =
        std::fs::read_to_string("Config.toml").expect("Unable to read config file");
    let config: Config = toml::from_str(&config_value).unwrap();
    config.discord
}

#[cfg(test)]
#[tokio::test]
async fn test_send_message_to_channel() {
    send_message_to_channel("Fuck you".to_string()).await;
}