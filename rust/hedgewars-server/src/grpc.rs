use mysql;
use mysql::{error::DriverError, error::Error, from_row_opt, params};
use platform::platform_client::PlatformClient;
use platform::GetProfilesRequest;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use tokio::process::Command;

pub mod platform {
    tonic::include_proto!("platform");
}

const INSERT_NEW_USER: &str =
    r"INSERT INTO users(uid, name, pass, status) VALUES (:id, :username, :password, 1)";

pub(crate) async fn listen_users() -> Result<(), Box<dyn std::error::Error>> {
    let pool = mysql::Pool::new(
        "mysql://hedgewars:2yB9OnKbYpYxBrQeguJOV4PJIrRafV@172.31.0.2:3306/hedgewars",
    )?;

    let docker_inspect_output = Command::new("docker")
        .arg("inspect")
        .arg("-f")
        .arg("{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}")
        .arg("platform")
        .output()
        .await?;

    let ip = String::from_utf8(docker_inspect_output.stdout)?;
    let endpoint = format!("http://{}:4000", ip.trim_end());
    let mut client = PlatformClient::connect(endpoint).await?;

    let request = GetProfilesRequest { streaming: true };

    let mut stream = client
        .get_profiles(tonic::Request::new(request))
        .await?
        .into_inner();

    while let Some(response) = stream.message().await? {
        for profile in &response.profiles {
            if !profile.admin {
                let mut name_prefix = profile.name.replace("_", "-");
                name_prefix = name_prefix.replace(" ", "-");
                name_prefix = name_prefix.replace(|c: char| !c.is_ascii(), "");

                let password: String = thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(8)
                    .map(char::from)
                    .collect();

                for i in 0..4 {
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
