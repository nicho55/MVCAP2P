use serde::{Deserialize, Serialize};

const SUPABASE_URL: &str = "https://cbnbmweqyezxejipisvf.supabase.co";
const SUPABASE_ANON_KEY: &str = "sb_publishable_JY_scW17hysS_cUCpnhz-g_49N_YGxr";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomEntry {
    pub id: i64,
    pub code: String,
    pub gm_name: String,
    pub room_url: String,
    pub created_at: String,
}

pub fn list_rooms() -> Result<Vec<RoomEntry>, String> {
    let url = format!("{SUPABASE_URL}/rest/v1/rooms?order=created_at.desc");
    let resp = ureq::get(&url)
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Authorization", &format!("Bearer {SUPABASE_ANON_KEY}"))
        .call()
        .map_err(|e| format!("{e:?}"))?;
    resp.into_body().read_json::<Vec<RoomEntry>>().map_err(|e| format!("{e:?}"))
}

pub fn create_room(code: &str, gm_name: &str, room_url: &str) -> Result<(), String> {
    let url = format!("{SUPABASE_URL}/rest/v1/rooms");
    let body = serde_json::json!({
        "code": code,
        "gm_name": gm_name,
        "room_url": room_url,
    });
    ureq::post(&url)
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Authorization", &format!("Bearer {SUPABASE_ANON_KEY}"))
        .header("Content-Type", "application/json")
        .send_json(&body)
        .map_err(|e| format!("{e:?}"))?;
    Ok(())
}

pub fn delete_room(code: &str) -> Result<(), String> {
    let url = format!("{SUPABASE_URL}/rest/v1/rooms?code=eq.{code}");
    ureq::delete(&url)
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Authorization", &format!("Bearer {SUPABASE_ANON_KEY}"))
        .call()
        .map_err(|e| format!("{e:?}"))?;
    Ok(())
}
