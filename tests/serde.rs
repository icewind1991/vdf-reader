#![allow(clippy::disallowed_names)]

use miette::{GraphicalReportHandler, GraphicalTheme};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::read_to_string;
use test_case::test_case;
use vdf_reader::entry::Table;
use vdf_reader::{from_entry, from_str};

#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
enum Expected {
    Types {
        fixed_array: [u8; 3],
        flex_array: Vec<f32>,
        tuple: (bool, u8),
        single: SingleOrTriple<f32>,
        triple: SingleOrTriple<f32>,
        single_int: SingleOrTriple<f32>,
        another_tuple: (u8, String, bool),
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
        env_map_tint: [f32; 3],
    },
    UserConfigData {
        #[serde(rename = "Steam")]
        steam: UserConfigDataSteam,
        #[serde(rename = "FriendsMainDialog")]
        friends_main_dialog: UserConfigDataFriendsMainDialog,
        #[serde(rename = "Servers")]
        servers: UserConfigDataServers,
    },
    Sprite {
        #[serde(rename = "$spriteorientation")]
        sprite_orientation: SpriteOrientation,
        #[serde(rename = "$spriteorigin")]
        sprite_origin: [f32; 2],
        #[serde(rename = "$basetexture")]
        base_texture: String,
        #[serde(rename = "$no_fullbright")]
        no_full_bright: bool,
    },
    EnumInMap {
        foo: EnumInMap,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnumInMap {
    Bar { a: bool },
    Foo { a: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SpriteOrientation {
    ParallelUpright,
    #[default]
    VpParallel,
    Oriented,
    VpParallelOriented,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum SingleOrTriple<T> {
    Single(T),
    Triple([T; 3]),
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
struct DialogPos {
    xpos: u32,
    ypos: u32,
    wide: u16,
    tall: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserConfigDataFriendsMainDialog {
    #[serde(flatten)]
    pos: DialogPos,
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
    #[serde(flatten)]
    pos: DialogPos,
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
#[test_case("tests/data/game_text.vmt")]
#[test_case("tests/data/enuminmap.vdf")]
#[test_case("tests/errors/unmatched.vdf")]
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

#[test_case("tests/data/toplevel.vdf")]
#[test_case("tests/data/concrete.vmt")]
#[test_case("tests/data/DialogConfigOverlay_1280x720.vdf")]
#[test_case("tests/data/serde_array_type.vdf")]
fn test_serde_table(path: &str) {
    let raw = read_to_string(path).unwrap();
    match from_str::<Table>(&raw) {
        Ok(result) => {
            insta::assert_ron_snapshot!(format!("table__{}", path), result);
        }
        Err(e) => {
            let handler = GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor());
            let mut out = String::new();
            handler.render_report(&mut out, &e).unwrap();
            insta::assert_snapshot!(format!("table__{}", path), out)
        }
    }
}

#[test_case("tests/data/concrete.vmt")]
#[test_case("tests/data/messy.vdf")]
#[test_case("tests/data/DialogConfigOverlay_1280x720.vdf")]
#[test_case("tests/data/serde_array_type.vdf")]
fn test_serde_from_table(path: &str) {
    let raw = read_to_string(path).unwrap();
    let result = Table::load_from_str(&raw).unwrap();

    let material: Expected = from_entry(result.into()).expect("table to material");
    insta::assert_ron_snapshot!(format!("table_to_material__{}", path), material);
}
