use std::{
    env,
    io::{Read, Seek, Write},
    sync::Arc,
};

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
    tracing::{debug, error, info},
    tracing_subscriber::FmtSubscriber,
    walkdir::{DirEntry, WalkDir},
};

lazy_static! {
    static ref DB_TICKETS: Arc<Db> = Arc::new(sled::open("db/tickets").unwrap());
    static ref DB_ACCOUNT: Arc<Db> = Arc::new(sled::open("db/account").unwrap());
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

// adapted from https://github.com/zip-rs/zip/blob/172f60fb9ae98450631e4a99a08bbadb7e3aa9da/examples/write_dir.rs
pub fn zip_dir<T>(
    dir: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = zip::write::FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o775);

    let mut buffer = Vec::new();
    for entry in dir {
        let path = entry.path();
        let name = path.strip_prefix(std::path::Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            zip.start_file_from_path(name, options)?;
            let mut f = std::fs::File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            zip.add_directory_from_path(name, options)?;
        }
    }

    zip.finish()?;
    Ok(())
}

pub fn backup_task() -> Result<String, Error> {
    let mut t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        .to_string();
    t.push_str("_backup.zip");
    let backup_path = std::path::Path::new(t.as_str());
    let f = std::fs::File::create(&backup_path)?;

    /*let walk = WalkDir::new("db");
    let mut dir = walk.into_iter().filter_map(|e| e.ok());
    let d = &mut dir;
    zip_dir(d, "db", f, zip::CompressionMethod::Deflated)?;*/

    let mut zip = zip::ZipWriter::new(f);
    zip.add_directory("db/", Default::default())?;

    let zip_opts = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    zip.start_file("db/tickets.txt", zip_opts)?;
    let tixs = std::fs::File::open("tickets.txt")?;
    let mut reader = std::io::BufReader::new(tixs);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    zip.write_all(&buffer)?;

    zip.start_file("db/account.txt", zip_opts)?;
    let accs = std::fs::File::open("account.txt")?;
    let mut reader = std::io::BufReader::new(accs);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    zip.write_all(&buffer)?;

    zip.finish()?;

    Ok(t)
}

pub fn export_tickets_db_tree() -> Result<(), Error> {
    let mut export = DB_TICKETS.export();
    let mut out = std::fs::File::create("tickets.txt")?;
    for (identifier, db_name, kv_iter) in export.drain(0..) {
        out.write_all(&identifier)?;
        out.write_all(&String::from("\n").as_bytes())?;
        out.write_all(&db_name)?;
        out.write_all(&String::from("\n").as_bytes())?;

        for kv in kv_iter {
            let mut counter = 0;
            for data in kv.into_iter() {
                out.write_all(&data)?;
                if counter == 0 {
                    out.write_all(&String::from(",").as_bytes())?;
                    counter += 1;
                    continue;
                }
                counter = 0;
                out.write_all(&String::from("\n").as_bytes())?;
            }
        }
    }
    out.sync_data()?;

    Ok(())
}

pub fn export_account_db_tree() -> Result<(), Error> {
    let mut export = DB_ACCOUNT.export();
    let mut out = std::fs::File::create("account.txt")?;
    for (identifier, db_name, kv_iter) in export.drain(0..) {
        out.write_all(&identifier)?;
        out.write_all(&String::from("\n").as_bytes())?;
        out.write_all(&db_name)?;
        out.write_all(&String::from("\n").as_bytes())?;

        for kv in kv_iter {
            let mut counter = 0;
            for data in kv.into_iter() {
                out.write_all(&data)?;
                if counter == 0 {
                    out.write_all(&String::from(",").as_bytes())?;
                    counter += 1;
                    continue;
                }
                counter = 0;
                out.write_all(&String::from("\n").as_bytes())?;
            }
        }
    }
    out.sync_data()?;

    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    // set tracing log subscriber
    let logger = FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(logger)
        .expect("setting default logging subscriber failed");

    // get environment variables

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

    let s3_access_key = env::var("S3_ACCESS_KEY").or_else(|e| {
        error!("s3 access key was not set in the environment: {e}");
        return Err(e);
    })?;
    let s3_secret_key = env::var("S3_SECRET_KEY").or_else(|e| {
        error!("s3 secret key was not set in the environment: {e}");
        return Err(e);
    })?;
    let s3_region = env::var("S3_REGION").or_else(|e| {
        error!("s3 region was not set in the environment: {e}");
        return Err(e);
    })?;
    let s3_endpoint = env::var("S3_ENDPOINT").or_else(|e| {
        error!("s3 region was not set in the environment: {e}");
        return Err(e);
    })?;

    let s3_bucket_name = env::var("S3_BUCKET_NAME").or_else(|e| {
        error!("s3 bucket name was not set in the environment: {e}");
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

    // spawn a backup task
    let s3_creds =
        s3::creds::Credentials::new(Some(&s3_access_key), Some(&s3_secret_key), None, None, None)?;
    let s3_region_custom = s3::Region::Custom {
        region: s3_region,
        endpoint: s3_endpoint,
    };
    let s3_bucket = s3::bucket::Bucket::new(&s3_bucket_name, s3_region_custom, s3_creds)?;
    std::thread::spawn(move || -> Result<(), Error> {
        loop {
            // sleep for 4 hours
            std::thread::sleep(std::time::Duration::from_secs_f64(60.0 * 60.0 * 4.0));

            info!("backup: running task!");

            // export database trees
            debug!("backup: exporting database trees");
            export_tickets_db_tree()?;
            export_account_db_tree()?;

            // compress to zip
            debug!("backup: compressing");
            let path = backup_task()?;

            // upload to s3
            let f = std::fs::File::open(&path)?;
            let mut reader = std::io::BufReader::new(f);
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer)?;
            s3_bucket.put_object(&path, &buffer)?;

            // clean up
            let _ = std::fs::remove_file("tickets.txt");
            let _ = std::fs::remove_file("account.txt");
            let _ = std::fs::remove_file(path);

            info!("backup: task finished!");
        }
    });

    // start it
    if let Err(e) = discord.start().await {
        error!("discord client did not start: {e}");
        // HACK: this error value is of a different type, so we'll panic instead.
        panic!("{}", e);
    }

    Ok(())
}
