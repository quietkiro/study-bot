mod commands;
mod db;
mod utils;

use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::env::var;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
#[allow(unused)]
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug)]
pub struct Data {
    pool: Pool<SqliteConnectionManager>,
    http: Arc<serenity::Http>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let manager = SqliteConnectionManager::file("data.db");
    let pool = Pool::new(manager).unwrap();
    db::startup(pool.clone());
    let token = var("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var, see README for more information.");
    let intents = serenity::GatewayIntents::non_privileged()
        // | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS;

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    pool: pool.clone(),
                    http: ctx.http.clone(),
                })
            })
        })
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            commands: vec![
                commands::profile(),
                commands::leaderboard(),
                commands::add_study_channel(),
                commands::remove_study_channel(),
            ],
            ..Default::default()
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}

async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        // serenity::FullEvent::Message { new_message } => {
        //     if new_message.content.to_lowercase().contains("poise")
        //         && new_message.author.id != ctx.cache.current_user().id
        //     {
        //         let old_mentions = data.poise_mentions.fetch_add(1, Ordering::SeqCst);
        //         new_message
        //             .reply(
        //                 ctx,
        //                 format!("Poise has been mentioned {} times", old_mentions + 1),
        //             )
        //             .await?;
        //     }
        // }
        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            // println!("old: {:#?}, new: {:#?}", old, new);
            let curr_time = SystemTime::now();
            let timestamp = match curr_time.duration_since(UNIX_EPOCH) {
                Ok(time) => time.as_secs(),
                Err(e) => {
                    println!("Failed to get time: {}", e);
                    0
                }
            };
            let user_id = new.member.as_ref().unwrap().user.id.get();
            let old_channel_id = match old {
                Some(v) => match v.channel_id {
                    Some(cid) => cid.get(),
                    None => 0,
                },
                None => 0,
            };
            let old_is_study_channel =
                old_channel_id != 0 && db::is_study_channel(data.pool.clone(), old_channel_id);
            let new_channel_id = match new.channel_id {
                Some(cid) => cid.get(),
                None => 0,
            };
            let new_is_study_channel =
                new_channel_id != 0 && db::is_study_channel(data.pool.clone(), new_channel_id);
            if !old_is_study_channel && new_is_study_channel {
                // joining a study channel (either moving in or connecting)
                db::log_start_time(data.pool.clone(), user_id, timestamp);
                println!("{} has started studying at {}", user_id, timestamp);
            } else if old_is_study_channel && !new_is_study_channel {
                // not joining a study channel (either moving away or disconnecting)
                db::log_end_time(data.pool.clone(), user_id, timestamp);
                println!("{} has stopped studying at {}", user_id, timestamp);
            }
        }
        _ => {}
    }
    Ok(())
}
