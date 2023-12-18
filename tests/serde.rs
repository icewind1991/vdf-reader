use miette::{GraphicalReportHandler, GraphicalTheme};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::read_to_string;
use test_case::test_case;
use vdf_reader::from_str;

#[derive(Debug, Serialize, Deserialize)]
enum Expected {
    Types {
        fixed_array: [u8; 3],
        flex_array: Vec<f32>,
        tuple: (bool, u8),
    },
    LightmappedGeneric {
        #[serde(rename = "$baseTexture")]
        base_texture: String,
        #[serde(rename = "$bumpmap")]
        bumpmap: String,
        #[serde(rename = "$ssbump")]
        ssbump: bool,
        #[serde(rename = "%keywords")]
        keywords: String,
        #[serde(rename = "$detail")]
        detail: String,
        #[serde(rename = "$detailscale")]
        detailscale: f32,
        #[serde(rename = "$detailblendmode")]
        detailblendmode: i32,
        #[serde(rename = "$detailblendfactor")]
        detailblendfactor: f32,
    },
    #[serde(rename = "Resource/specificPanel.res")]
    Messy {
        empty: (),
        array: Vec<u32>,
        windows_path: String,
        #[serde(rename = r#"\\"$translucent""#)]
        translucent: bool,
        #[serde(rename = "$envmaptint")]
        env_map_tint: f32,
        #[serde(rename = ".5")]
        spare: f32,
    },
    UserConfigData {
        #[serde(rename = "Steam")]
        steam: UserConfigDataSteam,
        #[serde(rename = "FriendsMainDialog")]
        friends_main_dialog: UserConfigDataFriendsMainDialog,
        #[serde(rename = "Servers")]
        servers: UserConfigDataServers,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct UserConfigDataSteam {
    cached: UserConfigDataSteamCached,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserConfigDataSteamCached {
    #[serde(rename = "OverlaySplash.res")]
    overlay_splash: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserConfigDataFriendsMainDialog {
    xpos: u32,
    ypos: u32,
    wide: u16,
    tall: u16,
    #[serde(rename = "FriendPanelSelf")]
    friends_panel_self: BTreeMap<String, String>,
    #[serde(rename = "FriendsDialogSheet")]
    friends_dialog_sheet: UserConfigDataFriendsMainDialogFriendsDialogSheet,
    #[serde(rename = "FriendsState")]
    friends_state: BTreeMap<String, u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserConfigDataFriendsMainDialogFriendsDialogSheet {
    #[serde(rename = "FriendsFriendsPage")]
    friends_friends_page: UserConfigDataFriendsMainDialogFriendsDialogSheetFriendsPage,
    #[serde(rename = "FriendsClansPage")]
    friends_clan_page: UserConfigDataFriendsMainDialogFriendsDialogSheetFriendsPage,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserConfigDataFriendsMainDialogFriendsDialogSheetFriendsPage {
    #[serde(rename = "BuddyList")]
    buddy_list: BTreeMap<String, bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserConfigDataServers {
    #[serde(rename = "DialogServerBrowser.res")]
    dialog_server_browser: UserConfigDataServersDialog,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserConfigDataServersDialog {
    xpos: u32,
    ypos: u32,
    wide: u16,
    tall: u16,
    #[serde(rename = "GameTabs")]
    game_tabs: UserConfigDataServersDialogGameTabs,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserConfigDataServersDialogGameTabs {
    #[serde(rename = "InternetGames")]
    internet_games: GameListHaver,
    #[serde(rename = "FavoriteGames")]
    favorite_games: GameListHaver,
    #[serde(rename = "HistoryGames")]
    history_games: GameListHaver,
    #[serde(rename = "SpectateGames")]
    spectate_games: GameListHaver,
    #[serde(rename = "LanGames")]
    lan_games: GameListHaver,
    #[serde(rename = "FriendsGames")]
    friends_games: GameListHaver,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameListHaver {
    gamelist: GameList,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameList {
    #[serde(rename = "#ServerBrowser_Password_hidden")]
    server_browser_password_hidden: bool,
    #[serde(rename = "#ServerBrowser_Bots_hidden")]
    server_browser_bots_hidden: bool,
    #[serde(rename = "#ServerBrowser_Secure_hidden")]
    server_browser_secure_hidden: bool,
    #[serde(rename = "#ServerBrowser_Servers_hidden")]
    server_browser_servers_hidden: bool,
    #[serde(rename = "#ServerBrowser_IPAddress_hidden")]
    server_browser_ip_address_hidden: bool,
    #[serde(rename = "#ServerBrowser_Game_hidden")]
    server_browser_game_hidden: bool,
    #[serde(rename = "#ServerBrowser_Players_hidden")]
    server_browser_players_hidden: bool,
    #[serde(rename = "#ServerBrowser_Map_hidden")]
    server_browser_map_hidden: bool,
    #[serde(rename = "#ServerBrowser_Latency_hidden")]
    server_browser_latency_hidden: bool,
    sort_column: String,
    sort_column_secondary: Option<String>,
    sort_column_asc: bool,
    sort_column_secondary_asc: bool,
}

#[test_case("tests/data/concrete.vmt")]
#[test_case("tests/data/messy.vdf")]
#[test_case("tests/data/DialogConfigOverlay_1280x720.vdf")]
#[test_case("tests/data/serde_array_type.vdf")]
#[test_case("tests/errors/concrete.vmt")]
#[test_case("tests/errors/novalue.vdf")]
#[test_case("tests/errors/serde_array_type.vdf")]
fn test_serde(path: &str) {
    let raw = read_to_string(path).unwrap();
    match from_str::<Expected>(&raw) {
        Ok(result) => insta::assert_ron_snapshot!(path, result),
        Err(e) => {
            let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor());
            let mut out = String::new();
            handler.render_report(&mut out, &e).unwrap();
            insta::assert_snapshot!(path, out)
        }
    }
}
