//! Requires the "client", "standard_framework", and "voice" features be enabled in your
//! Cargo.toml, like so:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["client", "standard_framework", "voice"]
//! ```
use std::env;

// This trait adds the `register_songbird` and `register_songbird_with` methods
// to the client builder below, making it easy to install this voice client.
// The voice client can be retrieved in any command using `songbird::get(ctx).await`.
use songbird::SerenityInit;

// Import the `Context` to handle commands.
use serenity::client::Context;

use serenity::{
    async_trait,
    client::{Client, EventHandler},
    framework::{
        StandardFramework,
        standard::{
            Args, CommandResult,
            macros::{command, group},
        },
    },
    model::{channel::Message, gateway::Ready},
    prelude::GatewayIntents,
    Result as SerenityResult,
};

struct Handler;

const HELLNET_ID: u64 = 350723544495292426u64;
const TEST_ID: u64 = 1016319781084856442u64;
const TEST_CHANNEL_ID: u64 = 1016319781844045888u64;
const TEST_VOICE_CHANNEL_ID: u64 = 1016319781844045889u64;

const GUILD_ID: u64 = TEST_ID;
const GUILD_CHANNEL_ID: u64 = TEST_CHANNEL_ID;
const GUILD_VOICE_CHANNEL_ID: u64 = TEST_VOICE_CHANNEL_ID;

const VIP_ID: u64 = 242359196803268610u64;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let guild_id = GUILD_ID;
        let guild_channel_id = GUILD_CHANNEL_ID;
        let guild_voice_channel_id = GUILD_VOICE_CHANNEL_ID;

        let channel = ctx.http.get_channel(guild_channel_id).await.unwrap();
        let channel_id = channel.id();
        let voice_channel = ctx.http.get_channel(guild_voice_channel_id).await.unwrap();
        let voice_channel_id = voice_channel.id();

        // Warten bis Brie im Voice Channel ist
        let mut guild = ctx.cache.guild(guild_id).unwrap();
        let member = guild.member(&ctx, VIP_ID).await.unwrap();
        let mut vip_voice_state = guild.voice_states.get(&member.user.id);
        while vip_voice_state.is_none() || vip_voice_state.unwrap().channel_id.unwrap() != voice_channel_id {
            println!("Waiting for Zinnusl to join the voice channel..., {}", vip_voice_state.is_none());
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            guild = ctx.cache.guild(guild_id).unwrap();
            vip_voice_state = guild.voice_states.get(&member.user.id);
        }

        // check_msg(channel_id.say(&ctx.http, "Es wird los fanfared!").await);

        let uri = "D:/Songs/Hartgefetzt.ogg";

        let manager = songbird::get(&ctx).await
            .expect("Songbird Voice client placed in at initialisation.").clone();

        let _handler = manager.join(guild_id, voice_channel_id).await;

        if let Some(handler_lock) = manager.get(guild_id) {
            let mut handler = handler_lock.lock().await;

            let source = match songbird::ffmpeg(&uri).await {
                Ok(source) => source,
                Err(why) => {
                    println!("Err starting source: {:?}", why);

                    check_msg(channel.id().say(&ctx.http, "Error sourcing ffmpeg").await);
                    return ();
                },
            };

            handler.play_source(source);
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| c
                   .prefix("~"));

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    tokio::spawn(async move {
        let _ = client.start().await.map_err(|why| println!("Client ended: {:?}", why));
    });
    
    tokio::signal::ctrl_c().await;
    println!("Received Ctrl-C, shutting down.");
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
