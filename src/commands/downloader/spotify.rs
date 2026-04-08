use crate::cmd;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    media: Media,
    title: String,
    artist: Vec<Artist>,
    cover: Vec<Cover>,
}

#[derive(Debug, Deserialize)]
struct Media {
    url: String,
}
#[derive(Debug, Deserialize)]
struct Artist {
    name: String,
}
#[derive(Debug, Deserialize)]
struct Cover {
    url: String,
}
cmd!(
    Spotify,
    name: "spotify",
    aliases: ["song", "play"],
    category: "downloader",
    execute: |ctx| {
        let mut search_client = ctx.state.spotify_search_client.write().await;
        let tracks = search_client.tracks(ctx.body, 1).await?;
        let mut track = String::new();
        if let Some(result) = tracks.first() {
            track = result.uri.clone();
        }
        let song_url = format!("https://open.spotify.com/track/{}", track.split(':').nth(2).unwrap_or(""));
        let response = ctx.state.http_client.get("https://chocomilk.amira.us.kg/v1/download/spotify?url=".to_string() + song_url.as_str()).send().await?;
        let resp: ApiResponse = response.json().await?;
        let artist_name = resp.data.artist
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        send_audio!(
            context: ctx,
            audio_data: resp.data.media.url,
            dst: ctx.info.source.chat,
            reply: true,
            config_context: |context_info: &mut waproto::whatsapp::ContextInfo| {
                context_info.external_ad_reply = Some(waproto::whatsapp::context_info::ExternalAdReplyInfo {
                    title: Some(resp.data.title),
                    body: Some(artist_name),
                    media_type: Some(1),
                    thumbnail_url: Some(resp.data.cover[0].url.clone()),
                    render_larger_thumbnail: Some(true),
                    ..Default::default()
                });
            }
        ).await?;

        println!("Done!");
    }
);
