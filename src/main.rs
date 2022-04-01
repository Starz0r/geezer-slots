use std::{env, sync::Arc};

use {
    anyhow::{Error, Result},
    lazy_static::lazy_static,
    rand::random,
    serenity::{
        async_trait,
        model::{
            gateway,
            id::{EmojiId, GuildId},
            interactions::{
                application_command::{
                    ApplicationCommand, ApplicationCommandInteractionDataOptionValue,
                    ApplicationCommandOptionType,
                },
                Interaction, InteractionResponseType,
            },
            prelude::*,
        },
        prelude::*,
        utils::{parse_emoji, MessageBuilder},
    },
    sled::Db,
    standard_dist::StandardDist,
    tracing::{debug, error, info, trace},
    tracing_subscriber::FmtSubscriber,
    walkdir::WalkDir,
};

lazy_static! {
    static ref DB_TICKETS: Arc<Db> = Arc::new(sled::open("tickets").unwrap());
    static ref DB_ACCOUNT: Arc<Db> = Arc::new(sled::open("account").unwrap());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, StandardDist)]
enum Pulls {
    #[weight(100)]
    VCommon,
    #[weight(95)]
    Common,
    #[weight(75)]
    Uncommon,
    #[weight(60)]
    Rare,
    #[weight(50)]
    VRare,
    #[weight(30)]
    Epic,
    #[weight(10)]
    Legendary,
    #[weight(1)]
    Jackpot,
}

struct AppHandler;

#[async_trait]
impl EventHandler for AppHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "pull" => slot_pull(ctx.http.clone(), *command.user.id.as_u64()).await,
                "units" => get_units(*command.user.id.as_u64()),
                _ => unreachable!("unimplemented command"),
            };

            if let Err(e) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                error!("slash command cannot be responded to: {e}");
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} has connected!", ready.user.name);
        let guild_id: _ = GuildId(
            env::var("GUILD_ID")
                .or_else(|e| {
                    error!("guild id was not set in the environment: {e}");
                    return Err(e);
                })
                .expect("")
                .parse()
                .or_else(|e| {
                    error!("guild id was not a valid unsigned integer: {e}");
                    return Err(e);
                })
                .expect(""),
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command
                        .name("pull")
                        .description("Pulls the slot machine lever.")
                })
                .create_application_command(|command| {
                    command
                        .name("units")
                        .description("Checks how many units you have.")
                })
        })
        .await;
    }
}

fn get_units(user: u64) -> String {
    let units: u64 = match DB_ACCOUNT.get(&user.to_string()) {
        Ok(val) => match val {
            Some(val) => String::from_utf8_lossy(&val.to_vec()).parse().unwrap(),
            None => 0, // the user had no account, so nothing
        },
        Err(e) => panic!("{}", e),
    };

    String::from("You have ".to_owned() + &units.to_string() + " Units.")
}

async fn slot_pull(http: Arc<serenity::http::client::Http>, user: u64) -> String {
    // get the user's ticket count
    let mut tickets: u64 = match DB_TICKETS.get(&user.to_string()) {
        Ok(val) => match val {
            Some(val) => String::from_utf8_lossy(&val.to_vec()).parse().unwrap(),
            None => 50, // the user had no tickets, they get 50 by default
        },
        Err(e) => panic!("{}", e),
    };

    // eat a ticket
    if tickets != 0 {
        tickets -= 1;
    } else {
        // if they have no tickets to eat, turn them away
        return String::from("❌Unfortunately, you do not have any tickets to perform pulls.");
    }
    DB_TICKETS
        .insert(&user.to_string(), tickets.to_string().as_bytes())
        .unwrap();

    // assemble emojis for message
    let guild_id: _ = env::var("GUILD_ID")
        .or_else(|e| {
            error!("guild id was not set in the environment: {e}");
            return Err(e);
        })
        .expect("")
        .parse()
        .or_else(|e| {
            error!("guild id was not a valid unsigned integer: {e}");
            return Err(e);
        })
        .expect("");
    let guild = http.get_guild(guild_id).await.unwrap();
    let emoji_default = guild.emojis.get(&EmojiId(808737149507076147)).unwrap();
    let emoji_thisdog = guild.emojis.get(&EmojiId(672248379023163392)).unwrap();
    let emoji_delfruit = guild.emojis.get(&EmojiId(951979442736074802)).unwrap();
    let emoji_bigface = guild.emojis.get(&EmojiId(269629753647038464)).unwrap();
    let emoji_miku = guild.emojis.get(&EmojiId(548647780500111390)).unwrap();
    let emoji_tagfacehd = guild.emojis.get(&EmojiId(476888451132686361)).unwrap();
    let emoji_patsball = guild.emojis.get(&EmojiId(378972419685351441)).unwrap();
    let emoji_fruitpride = guild.emojis.get(&EmojiId(562501619615268884)).unwrap();
    let emoji_mayumushi_ani = guild.emojis.get(&EmojiId(951798762567766036)).unwrap();
    let emoji_mayumushi = guild.emojis.get(&EmojiId(951906103271235654)).unwrap();
    let emoji_kidangry = guild.emojis.get(&EmojiId(269629941090484225)).unwrap();
    let emoji_kidsleeper = guild.emojis.get(&EmojiId(269629879949983758)).unwrap();
    let emoji_kidunamused = guild.emojis.get(&EmojiId(269629805111148554)).unwrap();
    let emoji_kidthinking = guild.emojis.get(&EmojiId(269629915882586122)).unwrap();
    let emoji_kidchamp = guild.emojis.get(&EmojiId(251469271077486602)).unwrap();
    let emoji_kidd = emoji_default;

    // perform a pull
    let pull: Pulls = random();
    let (units, emote, react) = match pull {
        Pulls::VCommon => (0, emoji_thisdog, emoji_kidangry),
        Pulls::Common => (25, emoji_delfruit, emoji_kidsleeper),
        Pulls::Uncommon => (100, emoji_bigface, emoji_kidunamused),
        Pulls::Rare => (250, emoji_miku, emoji_kidthinking),
        Pulls::VRare => (500, emoji_tagfacehd, emoji_kidchamp),
        Pulls::Epic => (1000, emoji_patsball, emoji_kidchamp),
        Pulls::Legendary => (2500, emoji_fruitpride, emoji_kidchamp),
        Pulls::Jackpot => (5000, emoji_mayumushi_ani, emoji_kidd),
    };

    // add units to user's count
    let account: u64 = match DB_ACCOUNT.get(&user.to_string()) {
        Ok(val) => match val {
            Some(val) => String::from_utf8_lossy(&val.to_vec()).parse().unwrap(),
            None => 0,
        },
        Err(e) => panic!("{}", e),
    };
    DB_ACCOUNT
        .insert(&user.to_string(), (account + units).to_string().as_bytes())
        .unwrap();

    // which row is the winning one
    use rand::Rng;
    let mut trng: rand::rngs::StdRng = rand::SeedableRng::from_entropy();
    let row: u8 = trng.gen_range(0..=2);

    // generate 6 random emojis for the rest
    let mut extra: Vec<&serenity::model::guild::Emoji> = Vec::new();
    let mut last_emote = emoji_default;
    let mut same = 0;
    loop {
        // get a emote
        let emote_rarity: Pulls = random();
        let emoji = match emote_rarity {
            Pulls::VCommon => emoji_thisdog,
            Pulls::Common => emoji_delfruit,
            Pulls::Uncommon => emoji_bigface,
            Pulls::Rare => emoji_miku,
            Pulls::VRare => emoji_tagfacehd,
            Pulls::Epic => emoji_patsball,
            Pulls::Legendary => emoji_fruitpride,
            Pulls::Jackpot => emoji_mayumushi,
        };
        // if is the same emote as the last one, increment counter
        if emoji.id == last_emote.id {
            same += 1;
        } else {
            same = 0;
            last_emote = emoji;
        }

        // if we hit the same emote 2 times in a row, we have to reroll
        if same == 2 {
            continue;
        }

        // otherwise push to extras
        extra.push(emoji);

        // stop once we have enough
        if extra.len() == 6 {
            break;
        };
    }

    tokio::join!(DB_TICKETS.flush_async(), DB_ACCOUNT.flush_async());

    let mut msg = MessageBuilder::new();

    if row == 0 {
        msg.push("|")
            .emoji(emote)
            .push("|")
            .emoji(emote)
            .push("|")
            .emoji(emote)
            .push("|\n");
        loop {
            msg.push("|").emoji(extra.pop().unwrap());
            if extra.len() == 3 {
                msg.push("|\n");
                break;
            }
        }
        loop {
            msg.push("|").emoji(extra.pop().unwrap());
            if extra.is_empty() {
                msg.push("|\n");
                break;
            }
        }
    } else if row == 1 {
        loop {
            msg.push("|").emoji(extra.pop().unwrap());
            if extra.len() == 3 {
                msg.push("|\n");
                break;
            }
        }
        msg.push("|")
            .emoji(emote)
            .push("|")
            .emoji(emote)
            .push("|")
            .emoji(emote)
            .push("|\n");

        loop {
            msg.push("|").emoji(extra.pop().unwrap());
            if extra.is_empty() {
                msg.push("|\n");
                break;
            }
        }
    } else {
        loop {
            msg.push("|").emoji(extra.pop().unwrap());
            if extra.len() == 3 {
                msg.push("|\n");
                break;
            }
        }
        loop {
            msg.push("|").emoji(extra.pop().unwrap());
            if extra.is_empty() {
                msg.push("|\n");
                break;
            }
        }
        msg.push("|")
            .emoji(emote)
            .push("|")
            .emoji(emote)
            .push("|")
            .emoji(emote)
            .push("|\n");
    }

    msg.emoji(react)
        .push(", Won ".to_owned() + &units.to_string() + " Units!")
        .build()
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    // set tracing log subscriber
    let logger = FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(logger)
        .expect("setting default logging subscriber failed");

    let tkn = env::var("DISCORD_TOKEN").or_else(|e| {
        error!("discord token was not set in the environment: {e}");
        return Err(e);
    })?;

    let app_id: u64 = env::var("APPLICATION_ID")
        .or_else(|e| {
            error!("application id was not set in the environment: {e}");
            return Err(e);
        })?
        .parse()
        .or_else(|e| {
            error!("application id was not a valid unsigned integer: {e}");
            return Err(e);
        })?;

    // build client
    let mut discord = Client::builder(tkn)
        .event_handler(AppHandler)
        .application_id(app_id)
        .await
        .or_else(|e| {
            error!("discord client could not be created: {e}");
            return Err(e);
        })?;

    // start it
    if let Err(e) = discord.start().await {
        error!("discord client did not start: {e}");
        // HACK: this error value is of a different type, so we'll panic instead.
        panic!("{}", e);
    }

    Ok(())
}
