use anyhow::Context as _;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use std::sync::LazyLock;
use std::time::Instant;

static LAST_DEPLOYMENT: LazyLock<Instant> = LazyLock::new(Instant::now);

static REQWEST_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent("TelegramBot")
        .build()
        .unwrap()
});

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

mod instadown;
mod tiklydown;

use instadown::instadown;
use tiklydown::TiklydownRsp;

// Check last deployment
#[poise::command(slash_command, prefix_command)]
async fn last_deploy(ctx: Context<'_>) -> Result<(), Error> {
    let elapsed = LAST_DEPLOYMENT.elapsed();
    let days = elapsed.as_secs() / 86400;
    let message = format!("Last deployment was {} days ago", days);
    ctx.say(message).await?;
    Ok(())
}

/// Help command, duh
#[poise::command(slash_command, prefix_command)]
async fn help(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    ctx.say(
        r#"[commands]
\# supports prefix & slash
help = shows this message
tt = gets cdn url from tiktok url. set true will make it send music url too."
ig = shitty cdn url grabber for ig reels # this shit barely works"#,
    )
    .await?;

    Ok(())
}

/// About bot
#[poise::command(slash_command, prefix_command)]
async fn info(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    ctx.say(
        r#"[bot]
description = "faustok but circumcised so i can host it for free on shuttle.rs"

[credits]
faustok.shuttle = <https://github.com/yuvlian/faustok-shuttle>
faustok.original = <https://github.com/yuvlian/faustok>
hosting = <https://shuttle.rs/>"#,
    )
    .await?;

    Ok(())
}

/// Get IG Reels CDN Url
#[poise::command(slash_command, prefix_command)]
async fn ig(ctx: Context<'_>, #[description = "IG Reel URL"] url: String) -> Result<(), Error> {
    ctx.defer().await?;

    let cdn_url = instadown(&url, &REQWEST_CLIENT).await?;
    ctx.say(format!("[this shit barely works sorry]({})", cdn_url))
        .await?;

    Ok(())
}

/// Get TikTok CDN Url
#[poise::command(slash_command, prefix_command)]
async fn tt(
    ctx: Context<'_>,
    #[description = "TikTok URL"] url: String,
    #[description = "Get Music"] set: Option<bool>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let t_rsp = TiklydownRsp::fetch_url(&url, &REQWEST_CLIENT).await?;
    let default_set = set.unwrap_or(false);
    let media = t_rsp.get_media_urls(default_set);

    let mut response = String::new();

    for (index, img) in media.0.iter().enumerate() {
        response.push_str(&format!("[Video/Image {}]({})\n", index + 1, img));
    }

    if let (true, Some(music_url)) = (default_set, media.1) {
        if !response.is_empty() {
            response.push('\n');
        }
        response.push_str(&format!("[Music]({})", music_url));
    }

    if !response.is_empty() {
        ctx.say(response).await?;
    } else {
        ctx.say("Couldn't find anything").await?;
    }

    Ok(())
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    println!("{:?}", *LAST_DEPLOYMENT);

    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![last_deploy(), ig(), tt(), help(), info()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(".".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let client = ClientBuilder::new(discord_token, intents)
        .framework(framework)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}
