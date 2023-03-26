#![windows_subsystem = "windows"]

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
use tokio::sync::Mutex;
use std::sync::Arc;

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
const HELLNET_CHANNEL_ID: u64 = 350723544495292427u64;
const HELLNET_VOICE_CHANNEL_ID: u64 = 767171780565139457u64;

const GUILD_ID: u64 = HELLNET_ID;
const GUILD_CHANNEL_ID: u64 = HELLNET_CHANNEL_ID;
const GUILD_VOICE_CHANNEL_ID: u64 = HELLNET_VOICE_CHANNEL_ID;

const HELLNET_CIV_VI_FORUM: u64 = 1081570084239192104u64;

const USER_AND_URI: [(u64, &str, &str); 2] = [
    // (242359196803268610u64, r#"D:/Songs/fanfare_zinnusl/brie_fanfare.wav"#, "Zinnusl"),
    (666745883605467157u64, r#"D:/Songs/fanfare_brie/brie_fanfare.wav"#, "Brie"),
    (331097633961672706u64, r#"D:/Songs/fanfare_fiona/brie_fanfare.wav"#, "Fiona"),
];

async fn wait_for_join(ctx: Arc<Mutex<Context>>, user_id: u64, fanfare_uri: &str, name: &str) -> SerenityResult<()> {
    let guild_id = GUILD_ID;

    // let voice_channel = ctx.lock().await.http.get_channel(guild_voice_channel_id).await.unwrap();
    {
        // Init cache?
        let channel = ctx.lock().await.http.get_channel(GUILD_CHANNEL_ID).await.unwrap();
        let _channel_id = channel.id();
    }
    let voice_channel_id = serenity::model::prelude::ChannelId::from(GUILD_VOICE_CHANNEL_ID);

    // Warten bis VIP im Voice Channel ist
    // TODO: Cache erneurn muss nur ein Thread
    loop {
        let mut guard = ctx.lock().await;
        let guild_result = guard.cache.guild(guild_id);
        let mut guild = match guild_result {
            Some(guild) => guild,
            None => {
                println!("Guild not found: {}. Init Cache?", guild_id);
                return Ok(());
            }
        };
        let member_result = guild.member(&*guard, user_id).await;
        let member = match member_result {
            Ok(member) => member,
            Err(_) => {
                println!("Member not found: {}. Init Cache?", user_id);
                return Ok(());
            },
        };
        let mut vip_voice_state = guild.voice_states.get(&member.user.id);
        let mut is_not_in_voice_channel = vip_voice_state.is_none() || vip_voice_state.unwrap().channel_id.unwrap() != voice_channel_id;
        while !is_not_in_voice_channel {
            println!("Waiting for {} to leave the voice channel..., {}", name, vip_voice_state.is_none());
            drop(guard);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            guard = ctx.lock().await;
            guild = guard.cache.guild(guild_id).unwrap();
            vip_voice_state = guild.voice_states.get(&member.user.id);
            is_not_in_voice_channel = vip_voice_state.is_none() || vip_voice_state.unwrap().channel_id.unwrap() != voice_channel_id;
        }
        is_not_in_voice_channel = vip_voice_state.is_none() || vip_voice_state.unwrap().channel_id.unwrap() != voice_channel_id;
        while is_not_in_voice_channel {
            println!("Waiting for {} to join the voice channel..., {}", name, vip_voice_state.is_none());
            drop(guard);
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            guard = ctx.lock().await;
            guild = guard.cache.guild(guild_id).unwrap();
            vip_voice_state = guild.voice_states.get(&member.user.id);
            is_not_in_voice_channel = vip_voice_state.is_none() || vip_voice_state.unwrap().channel_id.unwrap() != voice_channel_id;
        }

        let manager = songbird::get(&guard).await
            .expect("Songbird Voice client placed in at initialisation.").clone();

        let _handler = manager.join(guild_id, voice_channel_id).await;

        if let Some(handler_lock) = manager.get(guild_id) {
            let mut handler = handler_lock.lock().await;

            let source = match songbird::ffmpeg(&fanfare_uri).await {
                Ok(source) => source,
                Err(why) => {
                    println!("Err starting source: {:?}", why);

                    // check_msg(channel.id().say(&guard.http, "Error sourcing ffmpeg").await);
                    return Ok(());
                },
            };

            let playtime = source.metadata.duration.unwrap();
            let _ = handler.play_source(source);

            tokio::time::sleep(playtime).await;

            let _ = handler.leave().await;
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let ctx = Arc::new(Mutex::new(ctx));

        for user_and_uri in USER_AND_URI.iter() {
            let user = user_and_uri.0;
            let uri = user_and_uri.1;
            let name = user_and_uri.2;

            let clone = ctx.clone();
            tokio::spawn(async move {
                let _ = wait_for_join(clone, user, uri, name).await;
            });
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

    let intents = GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILD_PRESENCES | GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILDS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    tokio::spawn(async move {
        let _ = client.start().await.map_err(|why| println!("Client ended: {:?}", why));
    });
    
    let _ = tokio::signal::ctrl_c().await;
    println!("Received Ctrl-C, shutting down.");
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

