use anyhow::Context as _;
use poise::serenity_prelude::{ClientBuilder, GatewayIntents};
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

mod tiklydown;
use tiklydown::TiklydownRsp;

/// Help command, duh
#[poise::command(slash_command, prefix_command)]
async fn help(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    ctx.say(
        r#"[commands]
\# supports prefix & slash
help = shows this message
faustok = gets cdn url from tiktok url. set true will make it send music url too."#,
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

/// Get TikTok CDN Url
#[poise::command(slash_command, prefix_command)]
async fn faustok(
    ctx: Context<'_>,
    #[description = "TikTok URL"] url: String,
    #[description = "Get Music"] set: Option<bool>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let t_rsp = TiklydownRsp::fetch_url(&url).await?;
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
    // Get the discord token set in `Secrets.toml`
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![faustok(), help(), info()],
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
