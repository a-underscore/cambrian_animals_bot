use reqwest::Url;
use serenity::{
    async_trait,
    framework::standard::{
        help_commands,
        macros::{command, group, help},
        Args, CommandError, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    model::{
        channel::{ChannelType, Message},
        id::UserId,
    },
    prelude::*,
};
use std::{collections::HashSet, env, time::Duration};
use tokio::time;

const URL: &str = "https://en.wikipedia.org/wiki/Special:RandomInCategory/Extinct_animals";

#[group]
#[commands(ping, animal)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _data_about_bot: serenity::model::prelude::Ready) {
        if let Some(interval) = env::var("AUTO_MESSAGE_INTERVAL")
            .ok()
            .and_then(|i| i.parse::<u64>().ok())
        {
            let mut interval = time::interval(Duration::from_secs(interval));
            let general_channel_name = env::var("AUTO_MESSAGE_CHANNEL").expect("channel");
            let cache = &ctx.cache;
            let http = &ctx.http;

            loop {
                let _ = interval.tick().await;

                if let Ok(message) = get_animal().await {
                    for guild_id in cache.guilds() {
                        if let Ok(guild) = guild_id.to_partial_guild(&http).await {
                            if let Ok(channels) = guild.channels(http).await {
                                let general_channel_id = channels
                                    .values()
                                    .find(|channel| {
                                        if let ChannelType::Text = channel.kind {
                                            channel.name == general_channel_name
                                        } else {
                                            false
                                        }
                                    })
                                    .map(|channel| channel.id);

                                if let Some(channel_id) = general_channel_id {
                                    let _ = channel_id.say(&http, message.clone()).await;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("dotenv");

    let prefix = env::var("COMMAND_PREFIX").expect("prefix");
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(prefix))
        .group(&GENERAL_GROUP)
        .help(&MY_HELP);
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    loop {
        if let Err(why) = client.start().await {
            println!("GOD DAMN IT: {:?}", why);
        }
    }
}

async fn get_animal() -> Result<Url, CommandError> {
    let client = reqwest::Client::new();
    let response = client.get(URL).send().await?;
    let url = response.url();

    Ok(url.clone())
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "pong").await?;

    Ok(())
}

#[command]
async fn animal(ctx: &Context, msg: &Message) -> CommandResult {
    let url = get_animal().await?;

    msg.reply(ctx, format!("{url}")).await?;

    Ok(())
}

#[help]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await?;

    Ok(())
}
