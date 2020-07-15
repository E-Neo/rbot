use clap::{crate_version, App, AppSettings, Arg, SubCommand};
use rbot::{instagram, schedule, utsz};

#[tokio::main]
async fn main() {
    let matches = App::new("rbot")
        .version(crate_version!())
        .author("Yanxuan Cui <cuiyx18@mails.tsinghua.edu.cn>")
        .about("A collection of automatic tools")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("instagram")
                .about("Downloads Instagram user data annonymously")
                .arg(Arg::with_name("USERNAME").required(true))
                .arg(
                    Arg::with_name("HTTP_PROXY")
                        .long("http-proxy")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("OUTPUT")
                        .short("o")
                        .long("output")
                        .takes_value(true),
                ),
        )
        .subcommand(SubCommand::with_name("schedule").about("Schedule"))
        .subcommand(
            SubCommand::with_name("utsz")
                .about("Life in UTSZ")
                .arg(Arg::with_name("BUILDINGID").required(true))
                .arg(Arg::with_name("ROOMNAME").required(true)),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("instagram") {
        let mut builder =
            instagram::Builder::new(String::from(matches.value_of("USERNAME").unwrap()));
        if let Some(http_proxy) = matches.value_of("HTTP_PROXY") {
            builder = builder.http_proxy(http_proxy);
        }
        if let Some(output) = matches.value_of("OUTPUT") {
            builder = builder.save_path(output);
        }
        let mut downloader = builder.build();
        downloader.run().await.unwrap();
    } else if let Some(_) = matches.subcommand_matches("schedule") {
        schedule::Schedule::new().run().unwrap();
    } else if let Some(matches) = matches.subcommand_matches("utsz") {
        println!(
            "{}",
            utsz::electricity(
                matches.value_of("BUILDINGID").unwrap(),
                matches.value_of("ROOMNAME").unwrap()
            )
            .await
            .unwrap()
        )
    }
}
