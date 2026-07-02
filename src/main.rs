mod cli;
mod codex;
mod constants;
mod debug_server;
mod logger;
mod proto;
mod proxy_server;

#[cfg(feature = "frida-link")]
mod frida_ffi;
mod frida_server;

use std::sync::Arc;
use tokio::sync::broadcast;

use logger::Logger;

#[tokio::main]
async fn main() {
    let options = match cli::parse_cli_options() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let logger = Arc::new(Logger::new(options.debug_main, options.debug_frida));

    let (cdp_tx, _) = broadcast::channel::<String>(256);
    let (proxy_tx, _) = broadcast::channel::<String>(256);

    let debug_handle = {
        let options = options.clone();
        let logger = logger.clone();
        let cdp_tx = cdp_tx.clone();
        let proxy_rx = proxy_tx.subscribe();
        tokio::spawn(async move {
            debug_server::run_debug_server(options, logger, cdp_tx, proxy_rx).await;
        })
    };

    let proxy_handle = {
        let options = options.clone();
        let logger = logger.clone();
        let debug_tx = proxy_tx;
        let debug_rx = cdp_tx.subscribe();
        tokio::spawn(async move {
            proxy_server::run_proxy_server(options, logger, debug_tx, debug_rx).await;
        })
    };

    let frida_handle = {
        let logger = logger.clone();
        tokio::spawn(async move {
            frida_server::run_frida_server(options, logger).await;
        })
    };

    let _ = tokio::join!(debug_handle, proxy_handle, frida_handle);
}
