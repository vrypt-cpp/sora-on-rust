#[macro_export]
macro_rules! send_msg {
    ($ctx:expr, dst: $dst:expr, text: $text:expr, reply: $is_reply:expr) => {
        $crate::send_msg!($ctx.client, $ctx.msg, $ctx.info, $ctx.state, dst: $dst, text: $text, reply: $is_reply)
    };

    ($client:expr, $msg:expr, $info:expr, $state:expr, dst: $dst:expr, text: $text:expr, reply: $is_reply:expr) => {{
        let mut context = waproto::whatsapp::ContextInfo::default();
        let expiration = $state.get_expiration(&$dst.to_string());

        if $is_reply {
            let original_msg = &$msg;
            context = wacore::proto_helpers::build_quote_context_with_info(
                $info.id.clone(),
                &$info.source.sender.to_non_ad(),
                &$info.source.chat,
                &original_msg
            );
        }
        context.remote_jid = Some($info.source.chat.to_string());
        context.mentioned_jid = vec![$info.source.sender.to_non_ad().to_string()];
        if expiration > 0 {
            context.expiration = Some(expiration);
        }
       // println!("{:#?}", &context);
        
        let message = waproto::whatsapp::Message {
            extended_text_message: Some(Box::new(waproto::whatsapp::message::ExtendedTextMessage {
                text: Some($text.to_string()),
                context_info: Some(Box::new(context)),
                ..Default::default()
            })),
            ..Default::default()
        };


        // let debug_message = waproto::whatsapp::Message {
        //     extended_text_message: Some(Box::new(waproto::whatsapp::message::ExtendedTextMessage {
        //         text: Some(format!("{:#?}", message)),
        //         ..Default::default()
        //     })),
        //     ..Default::default()
        // };
        
        $client.send_message($dst.clone(), message)
        // $client.send_message($dst.clone(), debug_message)
    }};
}
