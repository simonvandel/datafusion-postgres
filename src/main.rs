use std::sync::Arc;

use pgwire::api::auth::noop::NoopStartupHandler;
use pgwire::api::query::PlaceholderExtendedQueryHandler;
use pgwire::api::{MakeHandler, StatelessMakeHandler};
use pgwire::tokio::process_socket;
use tokio::net::TcpListener;

mod datatypes;
mod handlers;

#[tokio::main]
async fn main() {
    let processor = Arc::new(StatelessMakeHandler::new(Arc::new(
        handlers::DfSessionService::new(),
    )));
    // We have not implemented extended query in this server, use placeholder instead
    let placeholder = Arc::new(StatelessMakeHandler::new(Arc::new(
        PlaceholderExtendedQueryHandler,
    )));
    let authenticator = Arc::new(StatelessMakeHandler::new(Arc::new(NoopStartupHandler)));

    let server_addr = "127.0.0.1:5432";
    let listener = TcpListener::bind(server_addr).await.unwrap();
    println!("Listening to {}", server_addr);
    println!("Execute SQL \"LOAD <path/to/json> <table name>;\" to load your data as table.");
    loop {
        let incoming_socket = listener.accept().await.unwrap();
        let authenticator_ref = authenticator.make();
        let processor_ref = processor.make();
        let placeholder_ref = placeholder.make();
        tokio::spawn(async move {
            process_socket(
                incoming_socket.0,
                None,
                authenticator_ref,
                processor_ref,
                placeholder_ref,
            )
            .await
        });
    }
}
