use crate::cmd;

cmd!(
    Ping,
    name: "ping",
    aliases: ["p"],
    category: "general",
    execute: |ctx| {
        ctx.reply("Pong!").await?;
    }
);