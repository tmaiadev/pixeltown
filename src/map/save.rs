use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};

use super::{Feature, Map, Terrain, TerrainType, MAP_SIZE};

const VERSION: u32 = 1;
const EXTENSION: &str = "save";
const USERDATA: &str = "userdata";
const PREFIX: &str = "pixeltown-";

#[derive(Deserialize)]
struct VersionProbe {
    version: u32,
}

#[derive(Serialize, Deserialize)]
struct SaveFile {
    version: u32,
    terrain_type: TerrainType,
    seed: u64,
    size: usize,
    terrain: Vec<String>,
    features: Vec<String>,
}

pub struct SaveEntry {
    pub path: PathBuf,
    pub modified: SystemTime,
}

impl SaveEntry {
    pub fn name(&self) -> &str {
        let stem = self.path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
        stem.strip_prefix(PREFIX).unwrap_or(stem)
    }
}

#[derive(Debug)]
pub enum SaveError {
    Io(io::Error),
    Format(serde_json::Error),
    UnsupportedVersion(u32),
    InvalidSize,
    InvalidTile(char),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SaveError::Io(err) => write!(f, "{err}"),
            SaveError::Format(err) => write!(f, "malformed save file: {err}"),
            SaveError::UnsupportedVersion(v) => write!(f, "unsupported save version {v}"),
            SaveError::InvalidSize => write!(f, "map must be {MAP_SIZE}x{MAP_SIZE} tiles"),
            SaveError::InvalidTile(c) => write!(f, "unknown tile '{c}'"),
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SaveError::Io(err) => Some(err),
            SaveError::Format(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for SaveError {
    fn from(err: io::Error) -> Self {
        SaveError::Io(err)
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(err: serde_json::Error) -> Self {
        SaveError::Format(err)
    }
}

fn terrain_char(terrain: Terrain, river: bool) -> char {
    match terrain {
        Terrain::Grass => '#',
        Terrain::Sand => '.',
        Terrain::Water if river => 'r',
        Terrain::Water => '~',
    }
}

fn parse_terrain(c: char) -> Result<(Terrain, bool), SaveError> {
    match c {
        '#' => Ok((Terrain::Grass, false)),
        '.' => Ok((Terrain::Sand, false)),
        '~' => Ok((Terrain::Water, false)),
        'r' => Ok((Terrain::Water, true)),
        _ => Err(SaveError::InvalidTile(c)),
    }
}

fn feature_char(feature: Option<Feature>) -> char {
    match feature {
        None => '-',
        Some(Feature::Tree) => 'T',
        Some(Feature::Forest) => 'F',
        Some(Feature::Rock) => 'R',
    }
}

fn parse_feature(c: char) -> Result<Option<Feature>, SaveError> {
    match c {
        '-' => Ok(None),
        'T' => Ok(Some(Feature::Tree)),
        'F' => Ok(Some(Feature::Forest)),
        'R' => Ok(Some(Feature::Rock)),
        _ => Err(SaveError::InvalidTile(c)),
    }
}

fn rows(cell: impl Fn(usize) -> char) -> Vec<String> {
    (0..MAP_SIZE)
        .map(|y| (0..MAP_SIZE).map(|x| cell(y * MAP_SIZE + x)).collect())
        .collect()
}

fn cells<T>(rows: &[String], parse: impl Fn(char) -> Result<T, SaveError>) -> Result<Vec<T>, SaveError> {
    if rows.len() != MAP_SIZE || rows.iter().any(|row| row.chars().count() != MAP_SIZE) {
        return Err(SaveError::InvalidSize);
    }
    rows.iter().flat_map(|row| row.chars()).map(parse).collect()
}

pub fn encode(map: &Map) -> String {
    let file = SaveFile {
        version: VERSION,
        terrain_type: map.terrain_type,
        seed: map.seed,
        size: MAP_SIZE,
        terrain: rows(|i| terrain_char(map.tiles[i], map.river[i])),
        features: rows(|i| feature_char(map.features[i])),
    };
    serde_json::to_string_pretty(&file).expect("save file serializes")
}

pub fn decode(text: &str) -> Result<Map, SaveError> {
    // Checked before the full parse, so a future layout reports its version instead of a format error.
    let probe: VersionProbe = serde_json::from_str(text)?;
    if probe.version != VERSION {
        return Err(SaveError::UnsupportedVersion(probe.version));
    }
    let file: SaveFile = serde_json::from_str(text)?;
    if file.size != MAP_SIZE {
        return Err(SaveError::InvalidSize);
    }
    let (tiles, river) = cells(&file.terrain, parse_terrain)?.into_iter().unzip();
    Ok(Map {
        terrain_type: file.terrain_type,
        seed: file.seed,
        tiles,
        river,
        features: cells(&file.features, parse_feature)?,
        roads: vec![None; MAP_SIZE * MAP_SIZE],
        power_lines: vec![None; MAP_SIZE * MAP_SIZE],
    })
}

pub fn file_name(kind: TerrainType, time: NaiveDateTime) -> String {
    let kind = kind.label().to_lowercase();
    format!("{PREFIX}{kind}-{}.{EXTENSION}", time.format("%Y-%m-%d_%H-%M-%S"))
}

// Lives next to the executable, e.g. `target/debug/userdata/` under cargo.
pub fn userdata_dir() -> io::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe.parent().ok_or_else(|| io::Error::other("executable has no parent folder"))?;
    Ok(dir.join(USERDATA))
}

fn write_to(dir: &Path, map: &Map, time: NaiveDateTime) -> Result<PathBuf, SaveError> {
    fs::create_dir_all(dir)?;
    let path = dir.join(file_name(map.terrain_type, time));
    // Written aside and renamed, so a crash mid-write never leaves a truncated save.
    let partial = path.with_extension("partial");
    fs::write(&partial, encode(map))?;
    fs::rename(&partial, &path)?;
    Ok(path)
}

pub fn write(map: &Map) -> Result<PathBuf, SaveError> {
    write_to(&userdata_dir()?, map, Local::now().naive_local())
}

pub fn read(path: &Path) -> Result<Map, SaveError> {
    decode(&fs::read_to_string(path)?)
}

pub fn delete(path: &Path) -> io::Result<()> {
    fs::remove_file(path)
}

fn list_in(dir: &Path) -> Vec<SaveEntry> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut saves: Vec<SaveEntry> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == EXTENSION))
        .filter_map(|entry| {
            let modified = entry.metadata().and_then(|m| m.modified()).ok()?;
            Some(SaveEntry { path: entry.path(), modified })
        })
        .collect();
    saves.sort_by_key(|save| std::cmp::Reverse(save.modified));
    saves
}

// Newest modified first.
pub fn list() -> Vec<SaveEntry> {
    userdata_dir().map(|dir| list_in(&dir)).unwrap_or_default()
}

pub fn latest() -> Option<PathBuf> {
    list().into_iter().next().map(|entry| entry.path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn time() -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 3, 7).unwrap().and_hms_opt(9, 5, 1).unwrap()
    }

    fn assert_same(a: &Map, b: &Map) {
        assert_eq!(a.terrain_type, b.terrain_type);
        assert_eq!(a.seed, b.seed);
        assert!(a.tiles == b.tiles);
        assert!(a.river == b.river);
        assert!(a.features == b.features);
    }

    #[test]
    fn encoding_round_trips_every_terrain_type() {
        for kind in TerrainType::ALL {
            let map = Map::generate(kind, 11);
            assert_same(&decode(&encode(&map)).unwrap(), &map);
        }
    }

    #[test]
    fn rivers_survive_the_round_trip() {
        let map = Map::generate(TerrainType::Inland, 3);
        assert!(map.river.iter().any(|&r| r));
        assert!(decode(&encode(&map)).unwrap().river == map.river);
    }

    #[test]
    fn encoded_map_is_one_line_per_row() {
        let text = encode(&Map::generate(TerrainType::Island, 1));
        assert!(text.contains("\"terrain_type\": \"island\""));
        let row_len = MAP_SIZE + 2;
        let rows = text.lines().filter(|l| l.trim().trim_end_matches(',').len() == row_len);
        assert_eq!(rows.count(), 2 * MAP_SIZE);
    }

    #[test]
    fn file_name_holds_terrain_type_and_timestamp() {
        assert_eq!(
            file_name(TerrainType::Archipelago, time()),
            "pixeltown-archipelago-2026-03-07_09-05-01.save"
        );
    }

    #[test]
    fn rejects_unknown_versions_sizes_and_tiles() {
        let text = encode(&Map::generate(TerrainType::Inland, 1));
        let bumped = text.replace("\"version\": 1", "\"version\": 99");
        assert!(matches!(decode(&bumped), Err(SaveError::UnsupportedVersion(99))));

        let mut file: SaveFile = serde_json::from_str(&text).unwrap();
        file.terrain.pop();
        let short = serde_json::to_string(&file).unwrap();
        assert!(matches!(decode(&short), Err(SaveError::InvalidSize)));

        let mut file: SaveFile = serde_json::from_str(&text).unwrap();
        file.features[0].replace_range(0..1, "?");
        let bad = serde_json::to_string(&file).unwrap();
        assert!(matches!(decode(&bad), Err(SaveError::InvalidTile('?'))));

        assert!(matches!(decode("not json"), Err(SaveError::Format(_))));

        let future = r#"{ "version": 2, "layers": [] }"#;
        assert!(matches!(decode(future), Err(SaveError::UnsupportedVersion(2))));
    }

    #[test]
    fn writes_into_a_folder_and_reads_back() {
        let dir = std::env::temp_dir().join(format!("pixeltown-save-test-{}", std::process::id()));
        let map = Map::generate(TerrainType::Seaside, 8);
        let path = write_to(&dir, &map, time()).unwrap();
        assert_eq!(path, dir.join("pixeltown-seaside-2026-03-07_09-05-01.save"));
        assert_same(&read(&path).unwrap(), &map);
        assert!(!path.with_extension("partial").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn lists_saves_newest_modified_first() {
        let dir = std::env::temp_dir().join(format!("pixeltown-list-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let now = SystemTime::now();
        for (name, age) in [("pixeltown-a.save", 30), ("pixeltown-b.save", 10), ("c.save", 20)] {
            let file = fs::File::create(dir.join(name)).unwrap();
            file.set_modified(now - std::time::Duration::from_secs(age)).unwrap();
        }
        fs::write(dir.join("notes.txt"), "").unwrap();

        let names: Vec<String> = list_in(&dir).iter().map(|e| e.name().to_owned()).collect();
        assert_eq!(names, ["b", "c", "a"]);
        assert!(list_in(&dir.join("missing")).is_empty());
        fs::remove_dir_all(dir).unwrap();
    }
}
