use crate::scenes::{
    gameplay::GameplayScene, intro::IntroScene, menu::MenuScene, terrain::TerrainScene, Scene,
};

const USAGE: &str = "Usage: pixeltown [--windowed] [--show-fps] [--scene <intro|menu|terrain|gameplay>]";

#[derive(Debug, Default, PartialEq)]
pub struct Args {
    pub windowed: bool,
    pub show_fps: bool,
    pub scene: Option<SceneName>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneName {
    Intro,
    Menu,
    Terrain,
    Gameplay,
}

impl SceneName {
    pub fn build(self) -> Box<dyn Scene> {
        match self {
            SceneName::Intro => Box::new(IntroScene::new()),
            SceneName::Menu => Box::new(MenuScene::new()),
            SceneName::Terrain => Box::new(TerrainScene::new()),
            SceneName::Gameplay => Box::new(GameplayScene::resume()),
        }
    }

    fn parse(name: &str) -> Result<Self, String> {
        match name {
            "intro" => Ok(SceneName::Intro),
            "menu" => Ok(SceneName::Menu),
            "terrain" => Ok(SceneName::Terrain),
            "gameplay" => Ok(SceneName::Gameplay),
            _ => Err(format!("unknown scene '{name}'")),
        }
    }
}

impl Args {
    pub fn from_env() -> Self {
        Self::parse(std::env::args().skip(1)).unwrap_or_else(|err| {
            eprintln!("error: {err}\n{USAGE}");
            std::process::exit(2);
        })
    }

    pub fn parse(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut parsed = Args::default();
        let mut args = args;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--windowed" => parsed.windowed = true,
                "--show-fps" => parsed.show_fps = true,
                "--scene" => {
                    let name = args.next().ok_or("--scene requires a value")?;
                    parsed.scene = Some(SceneName::parse(&name)?);
                }
                _ => return Err(format!("unknown argument '{arg}'")),
            }
        }
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Args, String> {
        Args::parse(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn defaults_to_fullscreen_intro() {
        assert_eq!(parse(&[]), Ok(Args::default()));
    }

    #[test]
    fn parses_windowed_and_scene() {
        let args = parse(&["--windowed", "--scene", "terrain"]).unwrap();
        assert!(args.windowed);
        assert_eq!(args.scene, Some(SceneName::Terrain));
    }

    #[test]
    fn parses_show_fps() {
        assert!(parse(&["--show-fps"]).unwrap().show_fps);
        assert!(!parse(&[]).unwrap().show_fps);
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse(&["--scene"]).is_err());
        assert!(parse(&["--scene", "nope"]).is_err());
        assert!(parse(&["--bogus"]).is_err());
    }
}
