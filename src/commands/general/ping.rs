use crate::cmd;
use tokio::net::TcpStream;
use std::time::Instant;

cmd!(
    Ping,
    name: "ping",
    aliases: ["p"],
    category: "general",
    execute: |ctx| {
        let server_wangsaf = "g.whatsapp.net:443";
        let start = Instant::now();
        match TcpStream::connect(server_wangsaf).await {
            Ok(_) => {
                let latency = start.elapsed();
                ctx.reply(&format!("```Pong! {}ms```", latency.as_millis())).await?;

            }
            Err(e) => {
                println!("Error connecting to {}: {}", server_wangsaf, e);
             }
         }
        });