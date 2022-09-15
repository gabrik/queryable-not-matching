use clap::Parser;
use futures::prelude::*;
use std::convert::TryFrom;
use zenoh::config::Config;
use zenoh::prelude::r#async::AsyncResolve;
use zenoh::prelude::*;
use futures::select;
use std::time::Duration;
use async_std::task::sleep;


#[derive(Debug, Parser)]
#[clap(name = "queriable")]
struct Opt {
    #[clap(short = 'l', long = "listen")]
    listen: String,
    #[clap(short = 'k', long = "keyexpr")]
    register_key_expr : String,
    #[clap(short = 'r', long = "reply-keyexpr")]
    reply_key_expr : String,

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

    config.listen.endpoints.extend(vec![args.listen.parse().unwrap()]);

    let session = zenoh::open(config).res().await.unwrap();

    let register_ke =  KeyExpr::try_from(&args.register_key_expr).unwrap();
    let reply_ke = KeyExpr::try_from(args.reply_key_expr).unwrap();

    let queryable = session.declare_queryable(&register_ke).res().await.unwrap();

    let value = format!("Hello from {}", args.register_key_expr);

    println!("Enter 'q' to quit...");
    let mut stdin = async_std::io::stdin();
    let mut input = [0_u8];

    loop {
        select!(
            query = queryable.recv_async() => {
                let query = query.unwrap();
                println!(">> [Queryable ] Received Query '{}'", query.selector());
                query.reply(Ok(Sample::new(reply_ke.clone(), value.clone()))).res().await.unwrap();
            },

            _ = stdin.read_exact(&mut input).fuse() => {
                match input[0] {
                    b'q' => break,
                    0 => sleep(Duration::from_secs(1)).await,
                    _ => (),
                }
            }
        );
    }

}