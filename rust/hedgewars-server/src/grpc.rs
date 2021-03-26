use mysql;
use mysql::{error::DriverError, error::Error, from_row_opt, params};
use platform::platform_client::PlatformClient;
use platform::GetProfilesRequest;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use tokio::process::Command;
use log::*;
use regex::Regex;

pub mod platform {
    tonic::include_proto!("platform");
}

const INSERT_NEW_USER: &str =
    r"INSERT INTO users(uid, name, pass, status) VALUES (:id, :username, :password, 1)";

pub(crate) async fn listen_users() -> Result<(), Box<dyn std::error::Error>> {
    let pool = mysql::Pool::new(
        "mysql://hedgewars:2yB9OnKbYpYxBrQeguJOV4PJIrRafV@hedgewars-db:3306/hedgewars",
    )?;

    let endpoint = format!("http://platform:4000");
    let mut client = PlatformClient::connect(endpoint).await?;

    let request = GetProfilesRequest { streaming: true };

    let mut stream = client
        .get_profiles(tonic::Request::new(request))
        .await?
        .into_inner();

    while let Some(response) = stream.message().await? {
        for profile in &response.profiles {
            if !profile.admin {
                let mut name_prefix = String::from(profile.name.trim());

                let re = Regex::new(r"[$()*+?\[\]^{|}\x7F\x00-\x1f]").unwrap();
                name_prefix = re.replace_all(&name_prefix, "").to_string();

                if name_prefix.len() > 40 {
                    name_prefix = String::from(&name_prefix[..40]);
                }

                name_prefix = name_prefix.replace("_", "-");
                name_prefix = name_prefix.replace(" ", "-");
                name_prefix = name_prefix.replace(|c: char| !c.is_ascii(), "");

                

                info!("Inserting user: {}", name_prefix);

                for i in 0..4 {
                    let password: String = thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(8)
                    .map(char::from)
                    .collect();

                    let username = format!("{}_{}", name_prefix, i.to_string());
                    let _ = pool
                        .first_exec(
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
