use crate::cmd;

cmd!(
    Debug,
    name: "debug",
    aliases: ["dmsg"],
    category: "tools",
    execute: |ctx| {
        if let Some(ext_msg) = &ctx.msg.extended_text_message
            && let Some(context_info) = &ext_msg.context_info {
                ctx.reply(format!("{:#?}", context_info.quoted_message).as_str()).await?;
            }
    }
);