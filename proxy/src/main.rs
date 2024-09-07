mod api;
mod config;
pub mod proxy;

use {
    crate::{api::EthServer, config::load_config},
    fast_log::{
        consts::LogSize,
        plugin::{file_split::RollingType, packer::LogPacker},
        Config, Logger,
    },
    jsonrpsee::server::{RpcModule, ServerBuilder},
    log::{info, LevelFilter, Log},
    proxy::Proxy,
    std::{env, net::SocketAddr},
    tokio::signal,
    tokio_util::sync::CancellationToken,
};

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let config_path = env::var("PROXY_CONFIG").expect("PROXY_CONFIG is not set");
    let config: config::Config = load_config(config_path).expect("load config error");
    let start_slot = config.start_slot.unwrap_or(0);

    let logger: &'static Logger = fast_log::init(
        Config::new()
            .console()
            .file_split(
                &config.log,
                LogSize::KB(512),
                RollingType::All,
                LogPacker {},
            )
            .level(LevelFilter::Info),
    )
    .expect("log init error");

    let url = config
        .host
        .parse::<SocketAddr>()
        .expect("incorrect host url");
    info!("Start proxy: local address {}", url);
    logger.flush();

    let rpc = ServerBuilder::default()
        .build(url)
        .await
        .expect("server start error");

    let token = CancellationToken::new();
    let proxy = Proxy::new(config, token.clone()).await;

    let mut module = RpcModule::new(());
    module
        .merge(EthServer::into_rpc(proxy.clone()))
        .expect("proxy impl error");

    let handle = rpc.start(module);

    tokio::select! {
        _ = proxy.start(start_slot) => {},
        _ = signal::ctrl_c() => {
            info!("Shutdown..");
            token.cancel();
        }
    }

    handle.stop().expect("server stop error");
}
