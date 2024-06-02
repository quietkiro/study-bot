use crate::db;
use crate::utils;

use crate::{Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, prefix_command)]
pub async fn profile(
    ctx: Context<'_>,
    #[description = "Select user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let mut title = "You have".to_string();
    let user = match user {
        Some(user) => {
            title = format!("{} has", user.name);
            user
        }
        None => ctx.author_member().await.unwrap().user.clone(),
    };
    let duration = db::get_total_study_duration(ctx.data().pool.clone(), user.id.get());
    let (h, m, s) = utils::get_hms(duration);
    let response = format!(
        "{} studied for a total of `{}`h `{}`m `{}`s",
        title, h, m, s
    );
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let result = db::get_leaderboard(ctx.data().pool.clone());
    let mut response = "**Leaderboard (Top 10)\n---**".to_string();
    for (user_id, duration) in result {
        let username = match serenity::UserId::new(user_id)
            .to_user(ctx.data().http.clone())
            .await
        {
            Ok(u) => u.name,
            _ => "###".to_string(),
        };
        let (h, m, s) = utils::get_hms(duration);
        let entry = format!("\n**{}**: `{}`h `{}`m `{}`s", username, h, m, s);
        response += &entry;
    }
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn add_study_channel(
    ctx: Context<'_>,
    #[description = "Channel to add"]
    #[channel_types("Voice")]
    channel: serenity::GuildChannel,
) -> Result<(), Error> {
    db::add_study_channel(ctx.data().pool.clone(), channel.id.get());
    ctx.say("Added study channel!").await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn remove_study_channel(
    ctx: Context<'_>,
    #[description = "Channel to remove"]
    #[channel_types("Voice")]
    channel: serenity::GuildChannel,
) -> Result<(), Error> {
    db::remove_study_channel(ctx.data().pool.clone(), channel.id.get());
    ctx.say("Removed study channel!").await?;
    Ok(())
}
