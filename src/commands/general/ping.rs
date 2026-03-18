use crate::cmd;

cmd!(
    Ping,
    name: "ping",
    aliases: ["p"],
    category: "general",
    execute: |client, _msg, info, state| {
        send_msg!(
            client, 
            info, 
            state,
            dst: info.source.chat, 
            text: "Pong!", 
            reply: true
        ).await?;
        
    }
);