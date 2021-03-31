use crate::core::types::GameCfg;
use crate::protocol::messages::HwProtocolMessage;
use crate::server::database::get_hash;
use base64::encode;
use lazy_static::lazy_static;
use log::*;
use mysql;
use mysql::{error::DriverError, error::Error, from_row_opt, params, Pool};
use platform::platform_client::PlatformClient;
use platform::{GetProfilesRequest, GetTasksRequest, Task};
use rand::{distributions::Alphanumeric, prelude::SliceRandom};
use rand::{thread_rng, Rng, RngCore};
use regex::Regex;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::io::BufWriter;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::time::{sleep, Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub mod platform {
    tonic::include_proto!("platform");
}

const INSERT_NEW_USER: &str =
    r"INSERT INTO users(`uid`, `name`, `pass`, `status`) VALUES (:id, :username, :password, 1)";
const REPLACE_USER: &str =
    r"REPLACE INTO users(`uid`, `name`, `pass`, `status`) VALUES (:id, :username, :password, 1)";
const GET_LEGALIZED_NAME: &str =
    r"SELECT LEFT(name, LENGTH(name) - 2) FROM users WHERE uid = :id LIMIT 1";
const REPLACE_MATCH: &str = r"REPLACE INTO matches(`nick`, `match`) VALUES (:nick, :match);";
const DELETE_MATCH: &str = r"DELETE FROM matches WHERE `match` = :match;";

lazy_static! {
    static ref MAPS: Vec<&'static str> = vec![
        "Battlefield",
        "Blizzard",
        "Blox",
        "Bubbleflow",
        "Cake",
        "Castle",
        "Cave",
        "Cheese",
        "Cogs",
        "Control",
        "CrazyMission",
        "EarthRise",
        "Hammock",
        "HedgeFortress",
        "Hedgelove",
        "Hogville",
        "Hydrant",
        "Islands",
        "Lonely_Island",
        "Mushrooms",
        "Octorama",
        "PirateFlag",
        "Plane",
        "SB_Bones",
        "SB_Crystal",
        "SB_Shrooms",
        "SB_Tentacles",
        "Sticks",
        "Trash",
        "Tree"
    ];
    static ref THEME_MAP: HashMap<String, String> = [
        ("Bamboo".to_string(), "Bamboo".to_string()),
        ("BambooPlinko".to_string(), "Bamboo".to_string()),
        ("Basketball".to_string(), "Nature".to_string()),
        ("BasketballField".to_string(), "Nature".to_string()),
        ("Bath".to_string(), "Bath".to_string()),
        ("Battlefield".to_string(), "Nature".to_string()),
        ("Blizzard".to_string(), "Snow".to_string()),
        ("Blox".to_string(), "Blox".to_string()),
        ("Bubbleflow".to_string(), "Underwater".to_string()),
        ("Cake".to_string(), "Cake".to_string()),
        ("Castle".to_string(), "Nature".to_string()),
        ("Cave".to_string(), "Island".to_string()),
        ("Cheese".to_string(), "Cheese".to_string()),
        ("ClimbHome".to_string(), "EarthRise".to_string()),
        ("Cogs".to_string(), "EarthRise".to_string()),
        ("Control".to_string(), "Deepspace".to_string()),
        ("CrazyMission".to_string(), "CrazyMission".to_string()),
        ("CTF_Blizzard".to_string(), "Snow".to_string()),
        ("EarthRise".to_string(), "EarthRise".to_string()),
        ("Eyes".to_string(), "Eyes".to_string()),
        ("Hammock".to_string(), "Nature".to_string()),
        ("HedgeFortress".to_string(), "Nature".to_string()),
        ("Hedgelove".to_string(), "Nature".to_string()),
        ("Hedgewars".to_string(), "Nature".to_string()),
        ("Hogville".to_string(), "Nature".to_string()),
        ("Hydrant".to_string(), "City".to_string()),
        ("Islands".to_string(), "Deepspace".to_string()),
        ("Knockball".to_string(), "Bamboo".to_string()),
        ("Lonely_Island".to_string(), "Island".to_string()),
        ("Mushrooms".to_string(), "Nature".to_string()),
        ("Octorama".to_string(), "Underwater".to_string()),
        ("PirateFlag".to_string(), "Island".to_string()),
        ("Plane".to_string(), "Planes".to_string()),
        ("portal".to_string(), "Hell".to_string()),
        ("Ropes".to_string(), "Eyes".to_string()),
        ("Ruler".to_string(), "Nature".to_string()),
        ("SB_Bones".to_string(), "Desert".to_string()),
        ("SB_Crystal".to_string(), "Cave".to_string()),
        ("SB_Grassy".to_string(), "Castle".to_string()),
        ("SB_Grove".to_string(), "Nature".to_string()),
        ("SB_Haunty".to_string(), "Halloween".to_string()),
        ("SB_Oaks".to_string(), "Nature".to_string()),
        ("SB_Shrooms".to_string(), "Nature".to_string()),
        ("SB_Tentacles".to_string(), "Hell".to_string()),
        ("Sheep".to_string(), "Sheep".to_string()),
        ("ShoppaKing".to_string(), "Castle".to_string()),
        ("Sticks".to_string(), "Bamboo".to_string()),
        ("Trash".to_string(), "Compost".to_string()),
        ("Tree".to_string(), "Halloween".to_string()),
        ("TrophyRace".to_string(), "Olympics".to_string())
    ]
    .iter()
    .cloned()
    .collect();
}

pub(crate) async fn listen_users() -> Result<(), Box<dyn std::error::Error>> {
    let pool = mysql::Pool::new(
        "mysql://hedgewars:2yB9OnKbYpYxBrQeguJOV4PJIrRafV@hedgewars-db:3306/hedgewars",
    )?;

    let endpoint = format!("http://platform_staging:4001");
    let mut client = PlatformClient::connect(endpoint).await?;

    let request = GetProfilesRequest { streaming: true };

    let mut stream = client
        .get_profiles(tonic::Request::new(request))
        .await?
        .into_inner();

    while let Some(response) = stream.message().await? {
        for profile in &response.profiles {
            if !profile.admin {
                let mut name_prefix = String::from(&profile.name);
                name_prefix = legalize_name(&name_prefix);

                info!("Inserting user: {}", name_prefix);

                for i in 0..4 {
                    let password: String = thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(8)
                        .map(char::from)
                        .collect();

                    let username = format!("{}_{}", name_prefix, i.to_string());
                    let _ = pool.first_exec(
                        INSERT_NEW_USER,
                        params! {
                            "id" => &profile.id,
                            "username" => &username,
                            "password" => &password
                        },
                    );
                }
            }
        }
    }

    Ok(())
}

pub(crate) async fn listen_tasks(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let pool = mysql::Pool::new(
        "mysql://hedgewars:2yB9OnKbYpYxBrQeguJOV4PJIrRafV@hedgewars-db:3306/hedgewars",
    )?;

    let password: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    let _ = pool.first_exec(
        REPLACE_USER,
        params! {
            "id" => "0",
            "username" => "STREAMBOT",
            "password" => &password
        },
    );
    println!("STREAMBOT created with password: {}", password);

    let endpoint = format!("http://platform_staging:4001");
    let mut client = PlatformClient::connect(endpoint).await?;

    let request = GetTasksRequest { streaming: true };

    let mut grpc_stream = client
        .get_tasks(tonic::Request::new(request))
        .await?
        .into_inner();

    sleep(Duration::from_millis(5000)).await;

    let mut task_handles: HashMap<String, tokio::task::JoinHandle<()>> = HashMap::new();
    let mut task_started = HashMap::new();
    let mut tasks_last_received: Vec<String> = Vec::new();

    while let Some(response) = grpc_stream.message().await? {
        let mut tasks_delted = tasks_last_received.clone();
        tasks_last_received = Vec::new();

        for task in &response.tasks {
            let pool = pool.clone();
            let task = task.clone();

            if task.name == "final-phase" {
                continue;
            }

            info!("Inserting room: {}", task.name.clone());
            tasks_last_received.push(task.name.clone());

            tasks_delted.retain(|name| name != &task.name.clone());

            let match_started = task_started.entry(task.name.clone())
                .or_insert(Arc::new(AtomicBool::new(false))).clone();
            if !match_started.load(Ordering::Relaxed) {
                if let Some(task_handle) = task_handles.remove(&task.name) {
                    task_handle.abort();
                }
            }

            let task_name = task.name.clone();
            let task_handle = tokio::spawn(async move {
                let res = send_to_hw_server(task, port, pool, match_started).await;
                if let Err(err) = res {
                    println!("worker failed: {:?}", err);
                }
            });
            task_handles.insert(task_name, task_handle);

            sleep(Duration::from_millis(500)).await;
        }

        for task_name in tasks_delted {
            let match_started = task_started.entry(task_name.clone())
                .or_insert(Arc::new(AtomicBool::new(false))).clone();
            if !match_started.load(Ordering::Relaxed) {
                if let Some(task_handle) = task_handles.remove(&task_name) {
                    task_handle.abort();
                }
            }
        }
    }

    Ok(())
}

async fn send_to_hw_server(task: Task, port: u16, pool: Pool, match_started: Arc<AtomicBool>) -> Result<(), std::io::Error> {
    let password: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    let username = format!("Backend_{}", task.name);

    let _ = pool.first_exec(
        REPLACE_USER,
        params! {
            "id" => "0",
            "username" => &username,
            "password" => &password
        },
    );

    let schedule_time = UNIX_EPOCH + Duration::from_secs(task.scheduled);
    let delay = schedule_time.duration_since(SystemTime::now()).unwrap_or_else(|err| {
        eprintln!("WEEEWOOOWEEEWOOO time stuff {:?} for match {}", err, task.name);
        Duration::from_secs(1)
    });
    let start = Instant::now() + delay;

    let mut hw_stream = TcpStream::connect(format!("localhost:{}", port)).await?;
    let (mut rx, mut tx) = hw_stream.split();

    let mut tx = BufWriter::new(tx);
    let mut rx = BufReader::new(rx);
    let mut lines = rx.lines();

    let mut room_created = false;

    macro_rules! send_msg {
        ($msg:expr) => {
            tx.write_all($msg.to_raw_protocol().as_bytes()).await?;
            tx.flush().await?;
        };
    }

    let mut teams: Vec<String> = Vec::new();
    let mut users_map: HashMap<String, bool> = HashMap::new();
    let mut clan_map: HashMap<String, u8> = HashMap::new();
    let mut nick_to_team_map: HashMap<String, String> = HashMap::new();
    for (i, team_id) in task.clone().teams.iter().enumerate() {
        let row = pool
            .first_exec(
                GET_LEGALIZED_NAME,
                params! {
                    "id" => &team_id
                },
            )
            .unwrap()
            .unwrap();
        let name = from_row_opt::<String>(row).unwrap();

        teams.push(name.clone());
        clan_map.insert(name.clone(), i as u8);
    }

    let sleep_scheduled_time = tokio::time::sleep_until(start);
    let sleep_force_start = tokio::time::sleep_until(start + Duration::from_secs(5*60));
    tokio::pin!(sleep_scheduled_time);
    tokio::pin!(sleep_force_start);

    'tcp_loop: loop {
        tokio::select! {
            _ = &mut sleep_scheduled_time, if !sleep_scheduled_time.is_elapsed() => {
                if match_started.load(Ordering::Relaxed) { continue; }
                send_msg!(HwProtocolMessage::Chat(format!("Waiting until either all players are ready or 5 more minutes...")));
            }
            _ = &mut sleep_force_start, if !sleep_force_start.is_elapsed() => {
                if match_started.load(Ordering::Relaxed) { continue; }
                send_msg!(HwProtocolMessage::StartGame);
                match_started.store(true, Ordering::Relaxed)
            }
            line = lines.next_line() => {
                let mut message = vec![line?.unwrap()];
                while let Some(arg) = lines.next_line().await? {
                    if arg == "" {
                        break;
                    }
                    message.push(arg);
                }

                let mut unknown = false;

                match &message.iter().map(|s| s.as_str()).collect::<Vec<_>>()[..] {
                    ["CONNECTED", text, protocol_version] => {
                        send_msg!(HwProtocolMessage::Nick(username.clone()));
                    }
                    ["NICK", nick] => {
                        send_msg!(HwProtocolMessage::Proto(1337));
                    }
                    ["PROTO", proto] => {}
                    ["ASKPASSWORD", server_salt] => {
                        let mut client_salt = [0u8; 18];
                        thread_rng().fill_bytes(&mut client_salt);
                        let client_salt = encode(&client_salt);

                        let client_hash = get_hash(1337, &password, &client_salt, &server_salt);
                        send_msg!(HwProtocolMessage::Password(
                            format!("{:x}", client_hash),
                            client_salt
                        ));
                    }
                    ["PING"] => {
                        send_msg!(HwProtocolMessage::Pong);
                    }
                    ["LOBBY:JOINED", nicks] => {
                        
                    }
                    ["ROOMS", infos@..] => {
                        if !room_created {
                            let name: &str = &task.name.clone();
                            if infos.contains(&name) {
                                send_msg!(HwProtocolMessage::JoinRoom(task.name.clone(), None));
                            } else {
                                send_msg!(HwProtocolMessage::CreateRoom(task.name.clone(), None));
                            }

                            pool.first_exec(
                                DELETE_MATCH,
                                params! {
                                    "match" => &task.name.clone(),
                                },
                            );
                            for team in teams.clone() {
                                let _ = pool.first_exec(
                                    REPLACE_MATCH,
                                    params! {
                                        "nick" => &team.clone(),
                                        "match" => &task.name.clone(),
                                    },
                                );
                            }

                            room_created = true;
                        }
                    }
                    ["ROOM", "ADD", flags, name, ..] => {}
                    ["JOINED", nicks@..] => {
                        let name: &str = &username.clone();
                        if nicks.contains(&name) {
                            send_msg!(HwProtocolMessage::Cfg(GameCfg::Ammo("Default".into(), Some("939192942219912103223511100120000000021110010101111100010000040504054160065554655446477657666666615551010111541111111070000000000000020550000004000700400000000022000000060002000000131111031211111112311411111111111111121111111111111111111110".into()))));
                            send_msg!(HwProtocolMessage::Cfg(GameCfg::Scheme(
                                "Default".into(),
                                vec![
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "true".into(),
                                    "false".into(),
                                    "false".into(),
                                    "true".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "false".into(),
                                    "100".into(),
                                    "45".into(),
                                    "100".into(),
                                    "15".into(),
                                    "5".into(),
                                    "3".into(),
                                    "4".into(),
                                    "0".into(),
                                    "2".into(),
                                    "0".into(),
                                    "0".into(),
                                    "35".into(),
                                    "25".into(),
                                    "47".into(),
                                    "5".into(),
                                    "100".into(),
                                    "100".into(),
                                    "0".into(),
                                    "!".into()
                                ]
                            )));
                            send_msg!(HwProtocolMessage::Cfg(GameCfg::Script("Normal".into())));
    
                            let map = get_random_map();
                            send_msg!(HwProtocolMessage::Cfg(GameCfg::Theme(
                                THEME_MAP[&map].to_string()
                            )));
                            send_msg!(HwProtocolMessage::Cfg(GameCfg::MapType(map)));
    
                            let seed: String = thread_rng()
                                .sample_iter(&Alphanumeric)
                                .take(32)
                                .map(char::from)
                                .collect();
    
                            send_msg!(HwProtocolMessage::Cfg(GameCfg::Seed(seed)));
                        }

                        send_msg!(HwProtocolMessage::Chat(
                            task.description.clone(),
                        ));

                        for nick in nicks {
                            if is_participant(nick.to_string(), teams.clone()) {
                                users_map.insert(nick.to_string(), false);
                            }
                        }
                    }
                    ["CLIENT_FLAGS", flags, nick] => {
                        if flags.contains("r") {
                            users_map.insert(nick.to_string(), flags.starts_with("+"));

                            if all_ready(teams.clone(), users_map.clone()) && task.teams.len() > 0 {
                                match_started.store(true, Ordering::Relaxed);
                                send_msg!(HwProtocolMessage::StartGame);
                            }
                        }
                    }
                    ["LEFT", nick, msg] => {
                        users_map.remove(&nick.to_string());
                    }
                    ["ADD_TEAM", team_name, grave, fort, voice_pack, flag, owner, difficulty, ..] => {
                        let mut owner_trim = owner.to_string();
                        owner_trim.pop();
                        owner_trim.pop();

                        if !clan_map.contains_key(&owner_trim) {
                            continue 'tcp_loop;
                        }

                        nick_to_team_map.insert(owner.to_string(), team_name.to_string());

                        send_msg!(HwProtocolMessage::SetTeamColor(
                            team_name.to_string(),
                            *clan_map.get(&owner_trim).unwrap()
                        ));

                        let clan_members: Vec<_> = nick_to_team_map
                            .iter()
                            .filter(|(nick, team_name)| nick.starts_with(&owner_trim))
                            .collect();

                        match clan_members.len() {
                            1 => {
                                for (owner, team_name) in clan_members.clone() {
                                    send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                        (&team_name).to_string(),
                                        4
                                    ));
                                }
                            }
                            2 => {
                                for (owner, team_name) in clan_members.clone() {
                                    send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                        (&team_name).to_string(),
                                        2
                                    ));
                                }
                            }
                            3 => {
                                for (i, (owner, team_name)) in clan_members.clone().iter().enumerate() {
                                    if i == 0 {
                                        send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                            (&team_name).to_string(),
                                            2
                                        ));
                                    } else {
                                        send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                            (&team_name).to_string(),
                                            1
                                        ));
                                    }
                                }
                            }
                            4 => {
                                for (owner, team_name) in clan_members.clone() {
                                    send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                        (&team_name).to_string(),
                                        1
                                    ));
                                }
                            }
                            _ => {}
                        }



                        if all_ready(teams.clone(), users_map.clone()) {
                            match_started.store(true, Ordering::Relaxed);
                            send_msg!(HwProtocolMessage::StartGame);
                        }
                    }
                    ["REMOVE_TEAM", team_name] => {
                        let mut owner_trim: String;
                        if let Some((owner, team)) = nick_to_team_map
                            .iter()
                            .find(|(owner, team)| team == team_name)
                        {
                            owner_trim = (*owner).clone();
                        } else {
                            continue 'tcp_loop;
                        }
                        owner_trim.pop();
                        owner_trim.pop();

                        nick_to_team_map.retain(|owner, nick| nick != team_name);

                        let clan_members: Vec<_> = nick_to_team_map
                            .iter()
                            .filter(|(nick, team_name)| nick.starts_with(&owner_trim.to_string()))
                            .collect();

                        match clan_members.len() {
                            1 => {
                                for (owner, team_name) in clan_members.clone() {
                                    send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                        (&team_name).to_string(),
                                        4
                                    ));
                                }
                            }
                            2 => {
                                for (owner, team_name) in clan_members.clone() {
                                    send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                        (&team_name).to_string(),
                                        2
                                    ));
                                }
                            }
                            3 => {
                                for (i, (owner, team_name)) in clan_members.clone().iter().enumerate() {
                                    if i == 0 {
                                        send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                            (&team_name).to_string(),
                                            2
                                        ));
                                    } else {
                                        send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                            (&team_name).to_string(),
                                            1
                                        ));
                                    }
                                }
                            }
                            4 => {
                                for (owner, team_name) in clan_members.clone() {
                                    send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                        (&team_name).to_string(),
                                        1
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }
                    ["LOBBY:LEFT", nick, ..] => {
                        users_map.remove(&nick.to_string());

                        let mut team_name: String;
                        let mut owner_trim: String;
                        if let Some((owner, team)) =
                            nick_to_team_map.iter().find(|(owner, team)| owner == nick)
                        {
                            owner_trim = (*owner).clone();
                            team_name = (*team).clone();
                        } else {
                            continue 'tcp_loop;
                        }
                        owner_trim.pop();
                        owner_trim.pop();

                        nick_to_team_map.retain(|owner, team| team != &team_name);

                        let clan_members: Vec<_> = nick_to_team_map
                            .iter()
                            .filter(|(nick, team_name)| nick.starts_with(&owner_trim.to_string()))
                            .collect();

                        match clan_members.len() {
                            1 => {
                                for (owner, team_name) in clan_members.clone() {
                                    send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                        (&team_name).to_string(),
                                        4
                                    ));
                                }
                            }
                            2 => {
                                for (owner, team_name) in clan_members.clone() {
                                    send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                        (&team_name).to_string(),
                                        2
                                    ));
                                }
                            }
                            3 => {
                                for (i, (owner, team_name)) in clan_members.clone().iter().enumerate() {
                                    if i == 0 {
                                        send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                            (&team_name).to_string(),
                                            2
                                        ));
                                    } else {
                                        send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                            (&team_name).to_string(),
                                            1
                                        ));
                                    }
                                }
                            }
                            4 => {
                                for (owner, team_name) in clan_members.clone() {
                                    send_msg!(HwProtocolMessage::SetHedgehogsNumber(
                                        (&team_name).to_string(),
                                        1
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }
                    message => {
                        info!("(Internal Client) unknown message: {}", message.join(" "));
                        unknown = true;
                    }
                }

                if !unknown {
                    //info!("(Internal Client): {}", message.join(" "));
                }
            }
        }
    }


    Ok(())
}

fn legalize_name(name: &String) -> String {
    let re = Regex::new(r"[$()*+?\[\]^{|}\x7F\x00-\x1f]").unwrap();

    let mut new_name = name.trim().to_string();
    new_name = re.replace_all(&new_name, "").to_string();
    new_name = new_name.replace(|c: char| !c.is_ascii(), "");
    new_name = new_name.replace("_", "-");
    new_name = new_name.replace(" ", "-");

    if new_name.len() > 40 {
        new_name = String::from(&new_name[..40]);
    }

    new_name
}

fn get_random_map() -> String {
    MAPS.choose(&mut rand::thread_rng()).unwrap().to_string()
}

fn is_participant(nick: String, teams: Vec<String>) -> bool {
    let mut legalized_nick = nick.clone();
    legalized_nick.pop();
    legalized_nick.pop();

    teams.iter().fold(false, |mut res, team| {
        res |= team == &legalized_nick;
        res
    })
}

fn all_ready(teams: Vec<String>, users_map: HashMap<String, bool>) -> bool {
    let mut all_joined = true;
    for team in teams.clone() {
        all_joined &= users_map.iter().fold(false, |mut res, (nick, ready)| {
            let mut n = nick.clone();
            n.pop();
            n.pop();

            res |= team == n;
            res
        });
    }

    let all_ready = users_map.iter().fold(true, |mut res, (nick, ready)| {
        res &= ready;
        res
    });

    all_ready && all_joined
}
