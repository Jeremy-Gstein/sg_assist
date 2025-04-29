use poise::serenity_prelude as serenity;

mod commands;

pub struct Data {} // User data, which is stored and accessible in all command invocations

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => poise::builtins::on_error(error).await.unwrap(), 
    }
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder().with_default_directive(tracing::Level::INFO.into()).from_env_lossy(),
        )
        .with_target(true)
        .init();



    let options = poise::FrameworkOptions {
        commands: vec![
                commands::updatesim(), commands::roster(), 
                commands::mykeys(), commands::vault(), 
                commands::keysdone(), commands::help(),
        ],
        on_error: |error| Box::pin(on_error(error)),
        ..Default::default()
    };


    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data{

                })
            })
        })
        .options(options)
        .build();
    

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");

    let intents = serenity::GatewayIntents::non_privileged() 
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::GUILD_MESSAGES;

 
    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
