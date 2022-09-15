use clap::Parser;
use std::convert::TryFrom;
use zenoh::config::Config;
use zenoh::prelude::r#async::AsyncResolve;
use zenoh::prelude::*;
use std::time::Duration;
use async_std::task::sleep;


#[derive(Debug, Parser)]
#[clap(name = "queriable")]
struct Opt {
    #[clap(short = 'k', long = "keyexpr")]
    key_expr : String,
}



#[async_std::main]
async fn main() {
    // Init logging
    env_logger::try_init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    ).unwrap_or_else(|_| log::warn!("`env_logger` already initialized"));

    let args = Opt::parse();

    let mut config = Config::default();
    config.set_mode(Some("peer".parse().unwrap())).unwrap();


    let key_expr =  KeyExpr::try_from(&args.key_expr).unwrap();
    let session = zenoh::open(config).res().await.unwrap();

    println!("Sleeping for scouting");
    sleep(Duration::from_secs(5)).await;

    let replies = session.get(&key_expr).res().await.unwrap();

    while let Ok(reply) = replies.recv_async().await {
        match reply.sample {
            Ok(sample) => println!(
                ">> Received ('{}': '{}')",
                sample.key_expr.as_str(),
                sample.value,
            ),
            Err(err) => println!(">> Received (ERROR: '{}')", String::try_from(&err).unwrap()),
        }
    }

}