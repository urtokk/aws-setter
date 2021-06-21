use clap::{
    crate_description,
    crate_name,
    crate_version,
    App,
    Arg,
    SubCommand
};

mod aws;
mod config;

fn main() {
    let home = std::env::var("HOME")
                    .expect("Could not find HOME variable");
    let std_config = format!("{}/.config/aws_setter.yml", home);

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .subcommand(SubCommand::with_name("list"))
        .subcommand(SubCommand::with_name("assume")
            .arg(Arg::with_name("profile")
            .short("p")
            .long("profile")
            .help("profile name to assume")
            .takes_value(true)
            .required(true)))
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .help("path to config file of aws setter")
            .default_value(std_config.as_str()))
        .get_matches();

    let setter_config = {
        let setter_result = config::AwsSetterConfig::load(
        matches
                .value_of("config")
                    .unwrap()
                );
        match setter_result {
            Ok(s) => s,
            Err(err) => {
                eprintln!("Could not load setter config: {}", err);
                std::process::exit(1);
            }
        }
    };

    let mut client = {
        let client_result = aws::AwsClient::new();
        match client_result {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Could not create client: {}", e);
                std::process::exit(2);
            }
        }
    };

    match matches.subcommand_name() {
        Some("list") => {
            let profiles = setter_config.list_profiles();
            println!("Profiles configured:");
            println!("");
            profiles.into_iter().for_each(|p| println!("{}", p));
        },
        Some("assume") => {
            let arg_matches = matches.subcommand_matches("assume").unwrap();
            let profile = arg_matches.value_of("profile").unwrap();
            client.assume(
                profile,
                setter_config.get_role(profile).expect("Role not found"),
                setter_config.email.as_str()
            ).map_err(|e| eprintln!("Error assuming role: {}", e)).ok();
        },
        _ => {
            matches.usage();
        },
    }
}
