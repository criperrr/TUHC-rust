use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::RwLock;

pub const DEFAULT_DISCORD_CLIENT_ID: &str = "826580095689326633";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiscordActivity {
    #[serde(alias = "activity_type")]
    pub activity_type: String,
    pub details: String,
    pub state: String,
    #[serde(alias = "large_image_key")]
    pub large_image_key: String,
    #[serde(alias = "large_image_text")]
    pub large_image_text: String,
    #[serde(alias = "small_image_key")]
    pub small_image_key: String,
    #[serde(alias = "small_image_text")]
    pub small_image_text: String,
    #[serde(alias = "start_timestamp")]
    pub start_timestamp: u64,
    #[serde(alias = "page_id")]
    pub page_id: Option<String>,
    #[serde(alias = "track_id")]
    pub track_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiscordStatus {
    pub enabled: bool,
    pub connected: bool,
    #[serde(alias = "client_id")]
    pub client_id: String,
    pub activity: Option<DiscordActivity>,
}

struct DiscordRpcInner {
    enabled: bool,
    connected: bool,
    client_id: String,
    session_start: u64,
    last_activity: Option<DiscordActivity>,
    nonce_counter: u64,
}

#[derive(Clone)]
pub struct DiscordRpcManager {
    inner: Arc<RwLock<DiscordRpcInner>>,
}

/// Resolves an MSPA page_id to its canonical `(chapter_or_act, icon_key)` pair.
pub fn resolve_story_chapter(page_id: &str) -> (&'static str, &'static str) {
    let trimmed = page_id.trim();
    match trimmed {
        "jb2_000000" => return ("Jailbreak", "jailbreak"),
        "mc0001" | "MC00001" => return ("Blood Spade", "blood_spade"),
        "pony" => return ("Homestuck Act 3", "homestuck"),
        "pony2" => return ("Homestuck Act 2", "homestuck"),
        "darkcage" => return ("Homestuck Act 6 Intermission 1", "homestuck"),
        "darkcage2" => return ("Homestuck Act 6 Act 3", "homestuck"),
        "dancestors" => return ("Homestuck Act 6 Intermission 3", "homestuck"),
        _ => {}
    }

    if trimmed.starts_with("rq") || trimmed.starts_with("RQ") || trimmed.contains('_') {
        return ("Ryanquest", "ryanquest");
    }

    if let Ok(p) = trimmed.parse::<u32>() {
        match p {
            1..=135 => ("Jailbreak", "jailbreak"),
            136..=218 => ("Bard Quest", "bard_quest"),
            219..=298 => ("Problem Sleuth Chapter 1", "problem_sleuth"),
            299..=365 => ("Problem Sleuth Chapter 2", "problem_sleuth"),
            366..=443 => ("Problem Sleuth Chapter 3", "problem_sleuth"),
            444..=522 => ("Problem Sleuth Chapter 4", "problem_sleuth"),
            523..=598 => ("Problem Sleuth Chapter 5", "problem_sleuth"),
            599..=679 => ("Problem Sleuth Chapter 6", "problem_sleuth"),
            680..=748 => ("Problem Sleuth Chapter 7", "problem_sleuth"),
            749..=823 => ("Problem Sleuth Chapter 8", "problem_sleuth"),
            824..=896 => ("Problem Sleuth Chapter 9", "problem_sleuth"),
            897..=975 => ("Problem Sleuth Chapter 10", "problem_sleuth"),
            976..=1052 => ("Problem Sleuth Chapter 11", "problem_sleuth"),
            1053..=1132 => ("Problem Sleuth Chapter 12", "problem_sleuth"),
            1133..=1211 => ("Problem Sleuth Chapter 13", "problem_sleuth"),
            1212..=1299 => ("Problem Sleuth Chapter 14", "problem_sleuth"),
            1300..=1380 => ("Problem Sleuth Chapter 15", "problem_sleuth"),
            1381..=1442 => ("Problem Sleuth Chapter 16", "problem_sleuth"),
            1443..=1509 => ("Problem Sleuth Chapter 17", "problem_sleuth"),
            1510..=1588 => ("Problem Sleuth Chapter 18", "problem_sleuth"),
            1589..=1655 => ("Problem Sleuth Chapter 19", "problem_sleuth"),
            1656..=1722 => ("Problem Sleuth Chapter 20", "problem_sleuth"),
            1723..=1819 => ("Problem Sleuth Chapter 21", "problem_sleuth"),
            1820..=1840 => ("Problem Sleuth Chapter 22", "problem_sleuth"),
            1841..=1892 => ("Problem Sleuth Epilogue", "problem_sleuth"),
            1893..=1900 => ("Homestuck Beta", "homestuck_beta"),
            1901..=2147 => ("Homestuck Act 1", "homestuck"),
            2148..=2658 => ("Homestuck Act 2", "homestuck"),
            2659..=3053 => ("Homestuck Act 3", "homestuck"),
            3054..=3257 => ("Homestuck Intermission", "homestuck"),
            3258..=3888 => ("Homestuck Act 4", "homestuck"),
            3889..=4525 => ("Homestuck Act 5 Act 1", "homestuck"),
            4526..=6010 => ("Homestuck Act 5 Act 2", "homestuck"),
            6011..=6012 => ("Homestuck Intermission 2", "homestuck"),
            6013..=6194 => ("Homestuck Act 6 Act 1", "homestuck"),
            6195..=6319 => ("Homestuck Act 6 Intermission 1", "homestuck"),
            6320..=6566 => ("Homestuck Act 6 Act 2", "homestuck"),
            6567..=6716 => ("Homestuck Act 6 Intermission 2", "homestuck"),
            6717..=7162 => ("Homestuck Act 6 Act 3", "homestuck"),
            7163 | 7208 | 7298 => ("Homestuck Openbound (Act 6 Intermission 3)", "homestuck"),
            7164..=7337 => ("Homestuck Act 6 Intermission 3", "homestuck"),
            7338..=7340 => ("Homestuck Act 6 Act 4", "homestuck"),
            7341..=7411 => ("Homestuck Act 6 Intermission 4", "homestuck"),
            7412..=7826 => ("Homestuck Act 6 Act 5", "homestuck"),
            7827..=8142 => ("Homestuck Act 6 Intermission 5", "homestuck"),
            8143..=8177 => ("Homestuck Act 6 Act 6 Act 1", "homestuck"),
            8178..=8374 => ("Homestuck Act 6 Act 6 Intermission 1", "homestuck"),
            8375..=8430 => ("Homestuck Act 6 Act 6 Act 2", "homestuck"),
            8431..=8752 => ("Homestuck Act 6 Act 6 Intermission 2", "homestuck"),
            8753..=8801 => ("Homestuck Act 6 Act 6 Act 3", "homestuck"),
            8802..=8820 => ("Homestuck Act 6 Act 6 Intermission 3", "homestuck"),
            8821..=8843 => ("Homestuck Act 6 Act 6 Act 4", "homestuck"),
            8844..=9308 => ("Homestuck Act 6 Act 6 Intermission 4", "homestuck"),
            9309..=9348 => ("Homestuck Act 6 Act 6 Act 5", "homestuck"),
            9349..=9986 => ("Homestuck Act 6 Act 6 Intermission 5", "homestuck"),
            9987 => ("Homestuck [S] Collide.", "homestuck"),
            9988..=10026 => ("Homestuck Act 6 Act 6 Act 6", "homestuck"),
            10027..=10030 => ("Homestuck Act 7", "homestuck"),
            _ => ("Homestuck", "homestuck"),
        }
    } else {
        ("Homestuck", "homestuck")
    }
}

/// Encodes a Discord Local IPC binary frame: `[opcode: u32 LE, length: u32 LE, json_bytes]`
pub fn encode_ipc_frame(opcode: u32, payload: &serde_json::Value) -> Vec<u8> {
    let json_bytes = serde_json::to_vec(payload).unwrap_or_default();
    let len = json_bytes.len() as u32;
    let mut frame = Vec::with_capacity(8 + json_bytes.len());
    frame.extend_from_slice(&opcode.to_le_bytes());
    frame.extend_from_slice(&len.to_le_bytes());
    frame.extend_from_slice(&json_bytes);
    frame
}

/// Decodes a Discord Local IPC binary frame header and JSON payload
pub fn decode_ipc_frame(bytes: &[u8]) -> Option<(u32, serde_json::Value)> {
    if bytes.len() < 8 {
        return None;
    }
    let opcode = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let len = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    if bytes.len() < 8 + len {
        return None;
    }
    let val = serde_json::from_slice(&bytes[8..8 + len]).ok()?;
    Some((opcode, val))
}

/// Discovers candidate Discord IPC Unix domain socket paths across standard, Flatpak, Snap, and temp dirs
pub fn candidate_socket_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(custom_sock) = std::env::var("TUHC_DISCORD_IPC_SOCKET") {
        let p = PathBuf::from(custom_sock);
        paths.push(p);
    }

    let mut base_dirs = Vec::new();
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        base_dirs.push(PathBuf::from(&xdg));
        base_dirs.push(PathBuf::from(&xdg).join("app/com.discordapp.Discord"));
        base_dirs.push(PathBuf::from(&xdg).join(".flatpak/com.discordapp.Discord/xdg-run"));
        base_dirs.push(PathBuf::from(&xdg).join("snap.discord"));
    }

    #[cfg(unix)]
    {
        let uid = unsafe { libc::getuid() };
        let run_user = PathBuf::from(format!("/run/user/{}", uid));
        if !base_dirs.contains(&run_user) {
            base_dirs.push(run_user.clone());
            base_dirs.push(run_user.join("app/com.discordapp.Discord"));
            base_dirs.push(run_user.join(".flatpak/com.discordapp.Discord/xdg-run"));
            base_dirs.push(run_user.join("snap.discord"));
        }
    }

    for env_key in ["TMPDIR", "TMP", "TEMP"] {
        if let Ok(dir) = std::env::var(env_key) {
            let p = PathBuf::from(dir);
            if !base_dirs.contains(&p) {
                base_dirs.push(p);
            }
        }
    }

    let tmp_dir = PathBuf::from("/tmp");
    if !base_dirs.contains(&tmp_dir) {
        base_dirs.push(tmp_dir);
    }

    for dir in base_dirs {
        for i in 0..10 {
            paths.push(dir.join(format!("discord-ipc-{}", i)));
        }
    }

    paths
}

impl DiscordRpcManager {
    pub fn new(enabled: bool) -> Self {
        let client_id = std::env::var("TUHC_DISCORD_CLIENT_ID")
            .unwrap_or_else(|_| DEFAULT_DISCORD_CLIENT_ID.to_string());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            inner: Arc::new(RwLock::new(DiscordRpcInner {
                enabled,
                connected: false,
                client_id,
                session_start: now,
                last_activity: None,
                nonce_counter: 0,
            })),
        }
    }

    pub async fn get_status(&self) -> DiscordStatus {
        let guard = self.inner.read().await;
        DiscordStatus {
            enabled: guard.enabled,
            connected: guard.connected,
            client_id: guard.client_id.clone(),
            activity: guard.last_activity.clone(),
        }
    }

    pub async fn set_enabled(&self, enabled: bool) -> DiscordStatus {
        let (client_id, nonce, last_act) = {
            let mut guard = self.inner.write().await;
            guard.enabled = enabled;
            guard.nonce_counter = guard.nonce_counter.saturating_add(1);
            if !enabled {
                guard.connected = false;
            }
            (
                guard.client_id.clone(),
                format!("tuhc-rpc-{}", guard.nonce_counter),
                guard.last_activity.clone(),
            )
        };

        if enabled {
            if let Some(ref act) = last_act {
                let connected = Self::try_emit_ipc(&client_id, Some(act), &nonce).await;
                self.inner.write().await.connected = connected;
            }
        } else {
            let _ = Self::try_emit_ipc(&client_id, None, &nonce).await;
        }

        self.get_status().await
    }

    pub async fn clear_presence(&self) -> DiscordStatus {
        let (enabled, client_id, nonce) = {
            let mut guard = self.inner.write().await;
            guard.last_activity = None;
            guard.nonce_counter = guard.nonce_counter.saturating_add(1);
            (
                guard.enabled,
                guard.client_id.clone(),
                format!("tuhc-rpc-{}", guard.nonce_counter),
            )
        };

        if enabled {
            let connected = Self::try_emit_ipc(&client_id, None, &nonce).await;
            self.inner.write().await.connected = connected;
        }

        self.get_status().await
    }

    pub async fn update_reading_presence(&self, page_id: &str, page_title: &str) -> DiscordStatus {
        let (chapter, icon_key) = resolve_story_chapter(page_id);
        let state_text = if page_title.trim().is_empty() {
            format!("Page {}", page_id)
        } else {
            format!("{} (Page {})", page_title.trim(), page_id)
        };

        let (enabled, client_id, nonce, activity) = {
            let mut guard = self.inner.write().await;
            guard.nonce_counter = guard.nonce_counter.saturating_add(1);
            let act = DiscordActivity {
                activity_type: "reading".to_string(),
                details: chapter.to_string(),
                state: state_text,
                large_image_key: icon_key.to_string(),
                large_image_text: chapter.to_string(),
                small_image_key: "mspa".to_string(),
                small_image_text: "The Unofficial Homestuck Collection".to_string(),
                start_timestamp: guard.session_start,
                page_id: Some(page_id.to_string()),
                track_id: None,
            };
            guard.last_activity = Some(act.clone());
            (
                guard.enabled,
                guard.client_id.clone(),
                format!("tuhc-rpc-{}", guard.nonce_counter),
                act,
            )
        };

        if enabled {
            let connected = Self::try_emit_ipc(&client_id, Some(&activity), &nonce).await;
            self.inner.write().await.connected = connected;
        }

        self.get_status().await
    }

    pub async fn update_music_presence(
        &self,
        track_title: &str,
        album_name: &str,
        artist: Option<&str>,
        track_id: Option<String>,
        playing: bool,
    ) -> DiscordStatus {
        let details = format!("Listening to {}", track_title.trim());
        let state_text = match artist.filter(|a| !a.trim().is_empty()) {
            Some(art) if !album_name.trim().is_empty() => {
                format!("{} — {}", art.trim(), album_name.trim())
            }
            Some(art) => art.trim().to_string(),
            None if !album_name.trim().is_empty() => album_name.trim().to_string(),
            None => "Homestuck Sound Test".to_string(),
        };

        let small_key = if playing { "play" } else { "pause" };
        let small_text = if playing { "Playing" } else { "Paused" };

        let (enabled, client_id, nonce, activity) = {
            let mut guard = self.inner.write().await;
            guard.nonce_counter = guard.nonce_counter.saturating_add(1);
            let act = DiscordActivity {
                activity_type: "music".to_string(),
                details,
                state: state_text.clone(),
                large_image_key: "music".to_string(),
                large_image_text: state_text,
                small_image_key: small_key.to_string(),
                small_image_text: small_text.to_string(),
                start_timestamp: guard.session_start,
                page_id: None,
                track_id,
            };
            guard.last_activity = Some(act.clone());
            (
                guard.enabled,
                guard.client_id.clone(),
                format!("tuhc-rpc-{}", guard.nonce_counter),
                act,
            )
        };

        if enabled {
            let connected = Self::try_emit_ipc(&client_id, Some(&activity), &nonce).await;
            self.inner.write().await.connected = connected;
        }

        self.get_status().await
    }

    #[cfg(unix)]
    async fn try_emit_ipc(
        client_id: &str,
        activity: Option<&DiscordActivity>,
        nonce: &str,
    ) -> bool {
        let candidates = candidate_socket_paths();
        for sock_path in candidates {
            if !sock_path.exists() {
                continue;
            }
            let res = tokio::time::timeout(
                std::time::Duration::from_millis(150),
                Self::send_over_unix_socket(&sock_path, client_id, activity, nonce),
            )
            .await;

            if let Ok(Ok(())) = res {
                return true;
            }
        }
        false
    }

    #[cfg(not(unix))]
    async fn try_emit_ipc(
        _client_id: &str,
        _activity: Option<&DiscordActivity>,
        _nonce: &str,
    ) -> bool {
        false
    }

    #[cfg(unix)]
    async fn send_over_unix_socket(
        sock_path: &std::path::Path,
        client_id: &str,
        activity: Option<&DiscordActivity>,
        nonce: &str,
    ) -> std::io::Result<()> {
        let mut stream = tokio::net::UnixStream::connect(sock_path).await?;

        // 1. Handshake (opcode 0)
        let handshake = serde_json::json!({
            "v": 1,
            "client_id": client_id,
        });
        let hs_frame = encode_ipc_frame(0, &handshake);
        stream.write_all(&hs_frame).await?;

        // Read handshake response header (non-fatal if empty, but standard Discord replies with READY)
        let mut header_buf = [0u8; 8];
        if stream.read_exact(&mut header_buf).await.is_ok() {
            let reply_len =
                u32::from_le_bytes([header_buf[4], header_buf[5], header_buf[6], header_buf[7]])
                    as usize;
            if reply_len <= 65536 {
                let mut body_buf = vec![0u8; reply_len];
                let _ = stream.read_exact(&mut body_buf).await;
            }
        }

        // 2. SET_ACTIVITY frame (opcode 1)
        let pid = std::process::id();
        let activity_json = match activity {
            Some(act) => serde_json::json!({
                "details": act.details,
                "state": act.state,
                "timestamps": {
                    "start": act.start_timestamp,
                },
                "assets": {
                    "large_image": act.large_image_key,
                    "large_text": act.large_image_text,
                    "small_image": act.small_image_key,
                    "small_text": act.small_image_text,
                }
            }),
            None => serde_json::Value::Null,
        };

        let cmd_payload = serde_json::json!({
            "cmd": "SET_ACTIVITY",
            "args": {
                "pid": pid,
                "activity": activity_json,
            },
            "nonce": nonce,
        });

        let cmd_frame = encode_ipc_frame(1, &cmd_payload);
        stream.write_all(&cmd_frame).await?;
        stream.flush().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_and_decode_ipc_frame() {
        let payload = serde_json::json!({
            "v": 1,
            "client_id": "826580095689326633"
        });
        let encoded = encode_ipc_frame(0, &payload);
        assert!(encoded.len() > 8);
        let (op, decoded) = decode_ipc_frame(&encoded).expect("Failed to decode IPC frame");
        assert_eq!(op, 0);
        assert_eq!(decoded["v"], 1);
        assert_eq!(decoded["client_id"], "826580095689326633");
    }

    #[test]
    fn test_resolve_story_chapter_mapping() {
        assert_eq!(resolve_story_chapter("000002"), ("Jailbreak", "jailbreak"));
        assert_eq!(
            resolve_story_chapter("jb2_000000"),
            ("Jailbreak", "jailbreak")
        );
        assert_eq!(
            resolve_story_chapter("000136"),
            ("Bard Quest", "bard_quest")
        );
        assert_eq!(
            resolve_story_chapter("000219"),
            ("Problem Sleuth Chapter 1", "problem_sleuth")
        );
        assert_eq!(
            resolve_story_chapter("001893"),
            ("Homestuck Beta", "homestuck_beta")
        );
        assert_eq!(
            resolve_story_chapter("001901"),
            ("Homestuck Act 1", "homestuck")
        );
        assert_eq!(
            resolve_story_chapter("002148"),
            ("Homestuck Act 2", "homestuck")
        );
        assert_eq!(
            resolve_story_chapter("pony"),
            ("Homestuck Act 3", "homestuck")
        );
        assert_eq!(
            resolve_story_chapter("003889"),
            ("Homestuck Act 5 Act 1", "homestuck")
        );
        assert_eq!(
            resolve_story_chapter("006009"),
            ("Homestuck Act 5 Act 2", "homestuck")
        );
        assert_eq!(
            resolve_story_chapter("007163"),
            ("Homestuck Openbound (Act 6 Intermission 3)", "homestuck")
        );
        assert_eq!(
            resolve_story_chapter("009987"),
            ("Homestuck [S] Collide.", "homestuck")
        );
        assert_eq!(
            resolve_story_chapter("010027"),
            ("Homestuck Act 7", "homestuck")
        );
        assert_eq!(resolve_story_chapter("rq0001"), ("Ryanquest", "ryanquest"));
    }

    #[tokio::test]
    async fn test_discord_rpc_manager_reading_and_music_states() {
        let rpc = DiscordRpcManager::new(true);

        let st1 = rpc.update_reading_presence("001901", "Homestuck").await;
        assert!(st1.enabled);
        let act1 = st1.activity.unwrap();
        assert_eq!(act1.activity_type, "reading");
        assert_eq!(act1.details, "Homestuck Act 1");
        assert_eq!(act1.state, "Homestuck (Page 001901)");
        assert_eq!(act1.large_image_key, "homestuck");
        assert_eq!(act1.page_id.as_deref(), Some("001901"));

        let st2 = rpc
            .update_music_presence(
                "Showtime (Piano Refrain)",
                "Homestuck Vol. 1",
                Some("Malcolm Brown"),
                Some("showtime-piano-refrain".to_string()),
                true,
            )
            .await;
        let act2 = st2.activity.unwrap();
        assert_eq!(act2.activity_type, "music");
        assert_eq!(act2.details, "Listening to Showtime (Piano Refrain)");
        assert_eq!(act2.state, "Malcolm Brown — Homestuck Vol. 1");
        assert_eq!(act2.small_image_key, "play");

        let st_off = rpc.set_enabled(false).await;
        assert!(!st_off.enabled);
        assert!(!st_off.connected);

        let st_cleared = rpc.clear_presence().await;
        assert!(st_cleared.activity.is_none());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_discord_rpc_unix_socket_wire_protocol() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sock_path = temp_dir.path().join("discord-ipc-0");
        let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();

        let server_task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut hs_header = [0u8; 8];
            stream.read_exact(&mut hs_header).await.unwrap();
            let hs_op =
                u32::from_le_bytes([hs_header[0], hs_header[1], hs_header[2], hs_header[3]]);
            let hs_len =
                u32::from_le_bytes([hs_header[4], hs_header[5], hs_header[6], hs_header[7]])
                    as usize;
            let mut hs_body = vec![0u8; hs_len];
            stream.read_exact(&mut hs_body).await.unwrap();
            let hs_json: serde_json::Value = serde_json::from_slice(&hs_body).unwrap();
            assert_eq!(hs_op, 0);
            assert_eq!(hs_json["v"], 1);

            // Reply with READY frame
            let ready =
                encode_ipc_frame(1, &serde_json::json!({"cmd": "DISPATCH", "evt": "READY"}));
            stream.write_all(&ready).await.unwrap();

            // Read SET_ACTIVITY frame
            let mut act_header = [0u8; 8];
            stream.read_exact(&mut act_header).await.unwrap();
            let act_op =
                u32::from_le_bytes([act_header[0], act_header[1], act_header[2], act_header[3]]);
            let act_len =
                u32::from_le_bytes([act_header[4], act_header[5], act_header[6], act_header[7]])
                    as usize;
            let mut act_body = vec![0u8; act_len];
            stream.read_exact(&mut act_body).await.unwrap();
            let act_json: serde_json::Value = serde_json::from_slice(&act_body).unwrap();
            assert_eq!(act_op, 1);
            assert_eq!(act_json["cmd"], "SET_ACTIVITY");
            assert_eq!(act_json["args"]["activity"]["details"], "Homestuck Act 1");
        });

        let act = DiscordActivity {
            activity_type: "reading".to_string(),
            details: "Homestuck Act 1".to_string(),
            state: "Homestuck (Page 001901)".to_string(),
            large_image_key: "homestuck".to_string(),
            large_image_text: "Homestuck Act 1".to_string(),
            small_image_key: "mspa".to_string(),
            small_image_text: "The Unofficial Homestuck Collection".to_string(),
            start_timestamp: 1700000000,
            page_id: Some("001901".to_string()),
            track_id: None,
        };

        DiscordRpcManager::send_over_unix_socket(
            &sock_path,
            DEFAULT_DISCORD_CLIENT_ID,
            Some(&act),
            "test-nonce-1",
        )
        .await
        .expect("Failed to send over mock Discord Unix socket");

        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_discord_rpc_offline_non_blocking_performance() {
        let rpc = DiscordRpcManager::new(true);

        // 1. Measure duration of 50 consecutive reading presence updates when offline
        let start = std::time::Instant::now();
        for i in 0..50 {
            let page_id = format!("{:06}", 1901 + i);
            let status = rpc.update_reading_presence(&page_id, "Homestuck").await;
            assert!(status.enabled);
            assert!(
                !status.connected,
                "Must be disconnected when no socket exists"
            );
        }
        let elapsed = start.elapsed();
        // 50 calls with no socket existing should finish in under 50ms (< 1ms each)
        assert!(
            elapsed.as_millis() < 500,
            "Offline presence updates must not block! Took {:?}",
            elapsed
        );

        // 2. Concurrently hammer RPC manager from 8 tasks
        let mut handles = Vec::new();
        for task_id in 0..8 {
            let rpc_clone = rpc.clone();
            handles.push(tokio::spawn(async move {
                for iter in 0..20 {
                    if iter % 4 == 0 {
                        let _ = rpc_clone.set_enabled(false).await;
                    } else if iter % 4 == 2 {
                        let _ = rpc_clone.set_enabled(true).await;
                    } else if iter % 2 == 1 {
                        let _ = rpc_clone
                            .update_music_presence(
                                "Sburban Jungle",
                                "Homestuck Vol. 1",
                                Some("Michael Guy Bowman"),
                                None,
                                true,
                            )
                            .await;
                    } else {
                        let _ = rpc_clone
                            .update_reading_presence(
                                &format!("00{:04}", task_id * 100 + iter),
                                "Test",
                            )
                            .await;
                    }
                }
            }));
        }

        for h in handles {
            h.await.unwrap();
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_discord_rpc_hanging_socket_timeout_resilience() {
        let temp_dir = tempfile::tempdir().unwrap();
        let sock_path = temp_dir.path().join("hanging-discord-ipc");
        let listener = tokio::net::UnixListener::bind(&sock_path).unwrap();

        // Spawn a hanging mock socket that accepts connection but never responds
        let hanging_server = tokio::spawn(async move {
            if let Ok((_stream, _)) = listener.accept().await {
                // Sleep indefinitely without replying to handshake
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });

        // Test send_over_unix_socket with timeout
        let act = DiscordActivity {
            activity_type: "reading".to_string(),
            details: "Homestuck Act 1".to_string(),
            state: "Homestuck (Page 001901)".to_string(),
            large_image_key: "homestuck".to_string(),
            large_image_text: "Homestuck Act 1".to_string(),
            small_image_key: "mspa".to_string(),
            small_image_text: "The Unofficial Homestuck Collection".to_string(),
            start_timestamp: 1700000000,
            page_id: Some("001901".to_string()),
            track_id: None,
        };

        let start = std::time::Instant::now();
        let res = tokio::time::timeout(
            std::time::Duration::from_millis(150),
            DiscordRpcManager::send_over_unix_socket(
                &sock_path,
                DEFAULT_DISCORD_CLIENT_ID,
                Some(&act),
                "hang-nonce",
            ),
        )
        .await;

        let elapsed = start.elapsed();
        assert!(res.is_err(), "Should time out on hanging socket");
        assert!(
            elapsed.as_millis() >= 140 && elapsed.as_millis() <= 350,
            "Timeout must trigger close to 150ms, elapsed: {:?}",
            elapsed
        );

        hanging_server.abort();
    }
}
