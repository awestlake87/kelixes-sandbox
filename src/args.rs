
use std::path::PathBuf;

use sc2::{ Launcher, LauncherSettings, CoordinatorSettings };
use sc2::data::{ GameSettings, Map };

use errors::{ Result };

pub const USAGE: &'static str = "
Tantrum StarCraft II Bot

Usage:
  tantrum (-h | --help)
  tantrum [options]
  tantrum --version

Options:
  -h --help                         Show this screen.
  --version                         Show version.
  --wine                            Use Wine to run StarCraft II (for Linux).
  -d <path> --dir=<path>            Path to the StarCraft II installation.
  -p <port> --port=<port>           Port to make StarCraft II listen on.
  -m <path> --map=<path>            Path to the StarCraft II map.
  -r --realtime                     Run StarCraft II in real time
  -s <count> --step-size=<count>    How many steps to take per call.
";
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
pub struct Args {
    pub flag_version:       bool,
    pub flag_wine:          bool,
    pub flag_dir:           Option<PathBuf>,
    pub flag_port:          Option<u16>,
    pub flag_map:           Option<PathBuf>,
    pub flag_realtime:      bool,
    pub flag_step_size:     Option<usize>,
}

pub fn get_coordinator_settings(args: &Args) -> Result<CoordinatorSettings> {
    let default_settings = LauncherSettings::default();

    let settings = LauncherSettings {
        use_wine: args.flag_wine,
        dir: args.flag_dir.clone(),
        base_port: match args.flag_port {
            Some(port) => port,
            None => default_settings.base_port
        }
    };

    Ok(
        CoordinatorSettings {
            launcher: Launcher::from(settings)?,
            replay_files: vec![ ],

            is_realtime: args.flag_realtime,
            step_size: match args.flag_step_size {
                Some(step_size) => step_size,
                None => 1
            }
        }
    )
}

pub fn get_game_settings(args: &Args) -> Result<GameSettings> {
    Ok(
        GameSettings {
            map: match args.flag_map {
                Some(ref map) => Map::LocalMap(map.clone()),
                None => bail!("no map specified")
            },
        }
    )
}
