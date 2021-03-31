use mysql::{self, from_row};
use mysql::{error::DriverError, error::Error, from_row_opt, params};
use openssl::sha::sha1;
use md5;

use crate::{core::{room::HwRoom, types::TeamInfo}, handlers::{AccountInfo, Sha1Digest}};

const CHECK_ACCOUNT_EXISTS_QUERY: &str =
    r"SELECT 1 FROM users WHERE users.name = :username LIMIT 1";

const GET_ACCOUNT_QUERY: &str = r"SELECT CASE WHEN users.status = 1 THEN users.pass ELSE '' END,
     (SELECT COUNT(users_roles.rid) FROM users_roles WHERE users.uid = users_roles.uid AND users_roles.rid = 3),
     (SELECT COUNT(users_roles.rid) FROM users_roles WHERE users.uid = users_roles.uid AND users_roles.rid = 13)
     FROM users WHERE users.name = :username";

const STORE_STATS_QUERY: &str = r"INSERT INTO gameserver_stats
      (players, rooms, last_update)
      VALUES
      (:players, :rooms, UNIX_TIMESTAMP())";

const GET_REPLAY_NAME_QUERY: &str = r"SELECT filename FROM achievements WHERE id = :id";
const GET_MATCH: &str = r"SELECT `nick` FROM matches WHERE `match` = :match";

pub struct ServerStatistics {
    rooms: u32,
    players: u32,
}

pub struct Achievements {}

pub struct Database {
    pool: Option<mysql::Pool>,
}

impl Database {
    pub fn new() -> Self {
        Self { pool: None }
    }

    pub fn connect(&mut self, url: &str) -> Result<(), Error> {
        self.pool = Some(mysql::Pool::new(url)?);

        Ok(())
    }

    pub fn is_registered(&mut self, nick: &str) -> Result<bool, Error> {
        if let Some(pool) = &self.pool {
            let nick_string = nick.to_string();
            let nick_trimmed = nick_string.split("_").collect::<Vec<&str>>()[0];

            let mut nick = nick;

            if nick_trimmed == "STREAMBOT" {
                nick = "STREAMBOT";
            }

            let is_registered = pool
                .first_exec(CHECK_ACCOUNT_EXISTS_QUERY, params! { "username" => nick })?
                .is_some();
            Ok(is_registered)
        } else {
            Err(DriverError::SetupError.into())
        }
    }

    pub fn get_account(
        &mut self,
        nick: &str,
        protocol: u16,
        password_hash: &str,
        client_salt: &str,
        server_salt: &str,
    ) -> Result<Option<AccountInfo>, Error> {
        if let Some(pool) = &self.pool {
            if protocol != 1337 {
                return Ok(None);
            }

            let nick_string = nick.to_string();
            let nick_trimmed = nick_string.split("_").collect::<Vec<&str>>()[0];

            let mut nick = nick;

            if nick_trimmed == "STREAMBOT" {
                nick = "STREAMBOT";
            }

            if let Some(row) = pool.first_exec(GET_ACCOUNT_QUERY, params! { "username" => nick })? {
                let (mut password, is_admin, is_contributor) =
                    from_row_opt::<(String, i32, i32)>(row)?;

                let client_hash = get_hash(protocol, &password, &client_salt, &server_salt);
                let server_hash = get_hash(protocol, &password, &server_salt, &client_salt);
                password.replace_range(.., "🦔🦔🦔🦔🦔🦔🦔🦔");

                if client_hash == password_hash {
                    Ok(Some(AccountInfo {
                        is_registered: true,
                        is_admin: is_admin == 1,
                        is_contributor: is_contributor == 1,
                        server_hash,
                    }))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Err(DriverError::SetupError.into())
        }
    }

    pub fn get_checker_account(
        &mut self,
        nick: &str,
        checker_password: &str,
    ) -> Result<bool, Error> {
        if let Some(pool) = &self.pool {
            if let Some(row) = pool.first_exec(GET_ACCOUNT_QUERY, params! { "username" => nick })? {
                let (mut password, _, _) = from_row_opt::<(String, i32, i32)>(row)?;
                Ok(checker_password == password)
            } else {
                Ok(false)
            }
        } else {
            Err(DriverError::SetupError.into())
        }
    }

    pub fn store_stats(&mut self, stats: &ServerStatistics) -> Result<(), Error> {
        if let Some(pool) = &self.pool {
            for mut stmt in pool.prepare(STORE_STATS_QUERY).into_iter() {
                stmt.execute(params! {
                    "players" => stats.players,
                    "rooms" => stats.rooms,
                })?;
            }
            Ok(())
        } else {
            Err(DriverError::SetupError.into())
        }
    }

    pub fn store_achievements(&mut self, achievements: &Achievements) -> Result<(), ()> {
        Ok(())
    }

    pub fn get_replay_name(&mut self, replay_id: u32) -> Result<Option<String>, Error> {
        if let Some(pool) = &self.pool {
            if let Some(row) =
                pool.first_exec(GET_REPLAY_NAME_QUERY, params! { "id" => replay_id })?
            {
                let filename = from_row_opt::<String>(row)?;
                Ok(Some(filename))
            } else {
                Ok(None)
            }
        } else {
            Err(DriverError::SetupError.into())
        }
    }

    pub fn get_match (
        &mut self,
        team_info: Box<TeamInfo>,
        room_name: &str,
        owner: &str,
        is_admin: &bool
    ) -> Result<bool, Error> {
        if let Some(pool) = &self.pool {
            let mut team_names: Vec<String> = Vec::new();

            for row in pool.prep_exec(GET_MATCH, params! { "match" => room_name })? {
                let team_name = from_row_opt::<String>(row?)?;
                team_names.push(team_name);
            }

            let mut nick_trimmed = owner.to_string();
            nick_trimmed.pop();
            nick_trimmed.pop(); 

            Ok(team_names.contains(&nick_trimmed.clone()) || *is_admin)
        } else {
            Err(DriverError::SetupError.into())
        }
    }
}

pub fn get_hash(protocol_number: u16, web_password: &str, salt1: &str, salt2: &str) -> Sha1Digest {
    let password = format!("{:x}", md5::compute(web_password));

    let s = format!(
        "{}{}{}{}{}",
        salt1, salt2, password, protocol_number, "!hedgewars"
    );
    Sha1Digest::new(sha1(s.as_bytes()))
}
