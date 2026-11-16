pub mod autotile;
pub mod buildings;
pub mod rng;
pub mod save;

use autotile::Wiring;
use buildings::{Building, BuildingKind, BuildingTile};
use rng::{fbm, Rng};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const MAP_SIZE: usize = 256;
const BORDER_MARGIN: usize = 8;
const EDGE_FALLOFF: f32 = 20.0;
const EDGE_SINK: f32 = 0.4;
const MAX_BEACH: u8 = 3;
const SEA_RATIO: f32 = 0.2;
const SEA_DEPTH_VARIATION: f32 = 16.0;
const FEATURE_SEED: u64 = 0xF0E5;
const FOREST_SCALE: f32 = 0.04;
const FOREST_THRESHOLD: f32 = 0.66;
const FOREST_FRINGE: f32 = 0.08;
const TREE_CHANCE: f32 = 0.04;
const FRINGE_TREE_CHANCE: f32 = 0.25;
const ROCK_CHANCE: f32 = 0.006;
const ROCK_SPREAD_CHANCE: f32 = 0.1;
const NEIGHBORS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Terrain {
    Grass,
    Water,
    Sand,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Feature {
    Tree,
    Forest,
    Rock,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axis {
    Horizontal,
    Vertical,
}

// A bridge is a road over water; it only runs straight along its axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Road {
    Street,
    Bridge(Axis),
}

// A power line bridge is a power line over water; it only runs straight along its axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerLine {
    Pole,
    Bridge(Axis),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TerrainType {
    Inland,
    Island,
    Archipelago,
    Seaside,
}

impl TerrainType {
    pub const ALL: [TerrainType; 4] = [
        TerrainType::Inland,
        TerrainType::Island,
        TerrainType::Archipelago,
        TerrainType::Seaside,
    ];

    pub fn label(self) -> &'static str {
        match self {
            TerrainType::Inland => "Inland",
            TerrainType::Island => "Island",
            TerrainType::Archipelago => "Archipelago",
            TerrainType::Seaside => "Seaside",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Side {
    North,
    South,
    West,
    East,
}

const SIDES: [Side; 4] = [Side::North, Side::South, Side::West, Side::East];

#[derive(Clone)]
pub struct Map {
    terrain_type: TerrainType,
    seed: u64,
    tiles: Vec<Terrain>,
    river: Vec<bool>,
    features: Vec<Option<Feature>>,
    roads: Vec<Option<Road>>,
    power_lines: Vec<Option<PowerLine>>,
    buildings: Vec<Option<BuildingTile>>,
}

impl Map {
    pub fn generate(kind: TerrainType, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let mut river = vec![false; MAP_SIZE * MAP_SIZE];
        let land = match kind {
            TerrainType::Inland => inland(&mut rng, &mut river),
            TerrainType::Island => island(seed),
            TerrainType::Archipelago => archipelago(seed),
            TerrainType::Seaside => seaside(&mut rng, seed, &mut river),
        };
        let tiles = add_beaches(&land, seed);
        let features = place_features(&tiles, seed);
        Self {
            terrain_type: kind,
            seed,
            tiles,
            river,
            features,
            roads: vec![None; MAP_SIZE * MAP_SIZE],
            power_lines: vec![None; MAP_SIZE * MAP_SIZE],
            buildings: vec![None; MAP_SIZE * MAP_SIZE],
        }
    }

    pub fn terrain(&self, x: usize, y: usize) -> Terrain {
        self.tiles[idx(x, y)]
    }

    pub fn is_river(&self, x: usize, y: usize) -> bool {
        self.river[idx(x, y)]
    }

    pub fn feature(&self, x: usize, y: usize) -> Option<Feature> {
        self.features[idx(x, y)]
    }

    pub fn road(&self, x: usize, y: usize) -> Option<Road> {
        self.roads[idx(x, y)]
    }

    pub fn power_line(&self, x: usize, y: usize) -> Option<PowerLine> {
        self.power_lines[idx(x, y)]
    }

    // The building covering this tile, by the top-left tile of its footprint.
    pub fn building_at(&self, x: usize, y: usize) -> Option<Building> {
        let tile = self.buildings[idx(x, y)]?;
        Some(Building { kind: tile.kind, x: (x - tile.dx as usize) as u16, y: (y - tile.dy as usize) as u16 })
    }

    // Row-major by top-left tile.
    pub fn buildings(&self) -> Vec<Building> {
        (0..MAP_SIZE * MAP_SIZE)
            .filter(|&i| matches!(self.buildings[i], Some(BuildingTile { dx: 0, dy: 0, .. })))
            .filter_map(|i| self.building_at(i % MAP_SIZE, i / MAP_SIZE))
            .collect()
    }

    // Buildings need dry, free ground; trees and forests do not count, they are cleared.
    pub fn can_build_building_on(&self, x: i32, y: i32) -> bool {
        if !Self::in_bounds(x, y) {
            return false;
        }
        let (x, y) = (x as usize, y as usize);
        self.terrain(x, y) != Terrain::Water
            && self.feature(x, y) != Some(Feature::Rock)
            && self.road(x, y).is_none()
            && self.power_line(x, y).is_none()
            && self.building_at(x, y).is_none()
    }

    // `center` is the tile the footprint is centered on.
    pub fn can_build_building(&self, kind: BuildingKind, center: (i32, i32)) -> bool {
        kind.tiles(kind.origin(center)).all(|(x, y)| self.can_build_building_on(x, y))
    }

    // Clears trees and forests under the footprint; returns whether the building was placed.
    pub fn build_building(&mut self, kind: BuildingKind, center: (i32, i32)) -> bool {
        if !self.can_build_building(kind, center) {
            return false;
        }
        let origin = kind.origin(center);
        for (x, y) in kind.tiles(origin) {
            let i = idx(x as usize, y as usize);
            let (dx, dy) = ((x - origin.0) as u8, (y - origin.1) as u8);
            self.buildings[i] = Some(BuildingTile { kind, dx, dy });
            self.features[i] = None;
        }
        true
    }

    // The axis a power line bridge placed on this water tile would run along: it must continue
    // a power line coming from a neighbor, and a neighbor that only wires along one axis can only be extended straight.
    fn power_bridge_axis(&self, x: usize, y: usize) -> Option<Axis> {
        let (x, y) = (x as i32, y as i32);
        [(0, -1), (1, 0), (0, 1), (-1, 0)].into_iter().find_map(|(dx, dy)| {
            let (nx, ny) = (x + dx, y + dy);
            if !Self::in_bounds(nx, ny) {
                return None;
            }
            let axis = if dx == 0 { Axis::Vertical } else { Axis::Horizontal };
            match autotile::wiring(self, nx as usize, ny as usize)? {
                Wiring::Span(a) if a != axis => None,
                _ => Some(axis),
            }
        })
    }

    // Power lines go over streets and on land; over water they need a neighboring power line and no bridge.
    pub fn can_build_power_line(&self, x: i32, y: i32) -> bool {
        if !Self::in_bounds(x, y) {
            return false;
        }
        let (x, y) = (x as usize, y as usize);
        if self.power_line(x, y).is_some() || self.building_at(x, y).is_some() || self.feature(x, y) == Some(Feature::Rock) {
            return false;
        }
        self.terrain(x, y) != Terrain::Water || (self.road(x, y).is_none() && self.power_bridge_axis(x, y).is_some())
    }

    // Replaces trees and forests; returns whether a power line was placed.
    pub fn build_power_line(&mut self, x: i32, y: i32) -> bool {
        if !self.can_build_power_line(x, y) {
            return false;
        }
        let (ux, uy) = (x as usize, y as usize);
        let line = match self.power_bridge_axis(ux, uy) {
            Some(axis) if self.terrain(ux, uy) == Terrain::Water => PowerLine::Bridge(axis),
            _ => PowerLine::Pole,
        };
        let i = idx(ux, uy);
        self.power_lines[i] = Some(line);
        self.features[i] = None;
        true
    }

    // The axis a bridge placed on this water tile would run along: it must continue a road
    // coming from a neighbor, and a neighboring bridge can only be extended straight.
    fn bridge_axis(&self, x: usize, y: usize) -> Option<Axis> {
        let (x, y) = (x as i32, y as i32);
        [(0, -1), (1, 0), (0, 1), (-1, 0)].into_iter().find_map(|(dx, dy)| {
            let (nx, ny) = (x + dx, y + dy);
            if !Self::in_bounds(nx, ny) {
                return None;
            }
            let axis = if dx == 0 { Axis::Vertical } else { Axis::Horizontal };
            match self.road(nx as usize, ny as usize)? {
                Road::Bridge(a) if a != axis => None,
                _ => Some(axis),
            }
        })
    }

    pub fn can_build_road(&self, x: i32, y: i32) -> bool {
        if !Self::in_bounds(x, y) {
            return false;
        }
        let (x, y) = (x as usize, y as usize);
        if self.road(x, y).is_some() || self.building_at(x, y).is_some() || self.feature(x, y) == Some(Feature::Rock) {
            return false;
        }
        self.terrain(x, y) != Terrain::Water || (self.power_line(x, y).is_none() && self.bridge_axis(x, y).is_some())
    }

    // Replaces trees and forests; returns whether a road was placed.
    pub fn build_road(&mut self, x: i32, y: i32) -> bool {
        if !self.can_build_road(x, y) {
            return false;
        }
        let i = idx(x as usize, y as usize);
        let road = match self.bridge_axis(x as usize, y as usize) {
            Some(axis) if self.terrain(x as usize, y as usize) == Terrain::Water => Road::Bridge(axis),
            _ => Road::Street,
        };
        self.roads[i] = Some(road);
        self.features[i] = None;
        true
    }

    pub fn can_bulldoze(&self, x: i32, y: i32) -> bool {
        Self::in_bounds(x, y) && {
            let (x, y) = (x as usize, y as usize);
            self.power_line(x, y).is_some()
                || self.road(x, y).is_some()
                || self.building_at(x, y).is_some()
                || self.feature(x, y).is_some()
        }
    }

    // Removes what is on top of the tile (a power line), else the road, else the whole building, else the tree, forest or rock; returns whether anything was removed.
    pub fn bulldoze(&mut self, x: i32, y: i32) -> bool {
        if !self.can_bulldoze(x, y) {
            return false;
        }
        let i = idx(x as usize, y as usize);
        if self.power_lines[i].take().is_some() || self.roads[i].take().is_some() {
            return true;
        }
        match self.building_at(x as usize, y as usize) {
            Some(building) => {
                for (bx, by) in building.tiles() {
                    self.buildings[idx(bx as usize, by as usize)] = None;
                }
            }
            None => self.features[i] = None,
        }
        true
    }

    pub fn in_bounds(x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < MAP_SIZE && (y as usize) < MAP_SIZE
    }

    pub fn get_or(&self, x: i32, y: i32, fallback: Terrain) -> Terrain {
        if Self::in_bounds(x, y) {
            self.terrain(x as usize, y as usize)
        } else {
            fallback
        }
    }
}

fn idx(x: usize, y: usize) -> usize {
    y * MAP_SIZE + x
}

fn center_distance(x: usize, y: usize) -> f32 {
    let half = MAP_SIZE as f32 / 2.0;
    let (dx, dy) = (x as f32 + 0.5 - half, y as f32 + 0.5 - half);
    (dx * dx + dy * dy).sqrt() / half
}

fn land_from_heights(heights: &[f32], ratio: f32) -> Vec<bool> {
    let mut sorted = heights.to_vec();
    sorted.sort_by(|a, b| b.total_cmp(a));
    let target = ((heights.len() as f32 * ratio) as usize).max(1);
    let threshold = sorted[target - 1];
    heights.iter().map(|&h| h >= threshold).collect()
}

fn height_field(seed: u64, height: impl Fn(usize, usize) -> f32) -> Vec<f32> {
    let mut heights = vec![f32::NEG_INFINITY; MAP_SIZE * MAP_SIZE];
    for y in BORDER_MARGIN..MAP_SIZE - BORDER_MARGIN {
        for x in BORDER_MARGIN..MAP_SIZE - BORDER_MARGIN {
            heights[idx(x, y)] = height(x, y) - edge_falloff(x, y, seed);
        }
    }
    heights
}

// Sinks land near the border so coasts curve away instead of being cut by the margin.
fn edge_falloff(x: usize, y: usize, seed: u64) -> f32 {
    let edge = x.min(y).min(MAP_SIZE - 1 - x).min(MAP_SIZE - 1 - y).saturating_sub(BORDER_MARGIN);
    let wobble = fbm(x as f32 * 0.02, y as f32 * 0.02, seed ^ 0xED6E, 2);
    let width = EDGE_FALLOFF * (0.5 + wobble);
    let t = 1.0 - (edge as f32 / width).min(1.0);
    t * t * EDGE_SINK
}

fn inland(rng: &mut Rng, river: &mut [bool]) -> Vec<bool> {
    let mut land = vec![true; MAP_SIZE * MAP_SIZE];
    let vertical = rng.chance(0.5);
    let flip = rng.chance(0.5);
    carve_river(&mut land, river, rng, vertical, flip);
    land
}

fn island(seed: u64) -> Vec<bool> {
    let heights = height_field(seed, |x, y| {
        let noise = fbm(x as f32 * 0.025, y as f32 * 0.025, seed, 4);
        (1.0 - center_distance(x, y)) + (noise - 0.5) * 0.6
    });
    land_from_heights(&heights, 0.65)
}

fn archipelago(seed: u64) -> Vec<bool> {
    let heights = height_field(seed, |x, y| {
        let noise = fbm(x as f32 * 0.03, y as f32 * 0.03, seed, 4);
        let mainland = (1.0 - center_distance(x, y) / 0.5).max(0.0).powf(1.2);
        noise * 0.7 + mainland * 0.3
    });
    land_from_heights(&heights, 0.5)
}

fn seaside(rng: &mut Rng, seed: u64, river: &mut [bool]) -> Vec<bool> {
    let mut land = vec![true; MAP_SIZE * MAP_SIZE];
    let sea = SIDES[rng.range(0, SIDES.len() as u32 - 1) as usize];
    let min_depth = MAP_SIZE as f32 * SEA_RATIO - SEA_DEPTH_VARIATION / 2.0;
    let depth: Vec<usize> = (0..MAP_SIZE)
        .map(|i| (min_depth + fbm(i as f32 * 0.04, 0.5, seed, 3) * SEA_DEPTH_VARIATION) as usize)
        .collect();

    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let is_sea = match sea {
                Side::North => y < depth[x],
                Side::South => y > MAP_SIZE - 1 - depth[x],
                Side::West => x < depth[y],
                Side::East => x > MAP_SIZE - 1 - depth[y],
            };
            if is_sea {
                land[idx(x, y)] = false;
            }
        }
    }

    if rng.chance(0.5) {
        let vertical = matches!(sea, Side::North | Side::South);
        let flip = matches!(sea, Side::North | Side::West);
        carve_river(&mut land, river, rng, vertical, flip);
    }
    land
}

fn river_point(vertical: bool, flip: bool, along: usize, across: i32) -> (i32, i32) {
    let a = if flip { MAP_SIZE - 1 - along } else { along } as i32;
    if vertical {
        (across, a)
    } else {
        (a, across)
    }
}

fn carve_river(
    land: &mut [bool],
    river: &mut [bool],
    rng: &mut Rng,
    vertical: bool,
    flip: bool,
) {
    let quarter = MAP_SIZE as u32 / 4;
    let mut across = rng.range(quarter, MAP_SIZE as u32 - quarter) as f32;
    let mut drift = 0.0f32;
    let half_width = 2;

    for along in 0..MAP_SIZE {
        drift = (drift + (rng.next_f32() - 0.5) * 0.6).clamp(-1.0, 1.0);
        across = (across + drift).clamp(4.0, MAP_SIZE as f32 - 5.0);
        for offset in -half_width..=half_width {
            let (x, y) = river_point(vertical, flip, along, across as i32 + offset);
            if Map::in_bounds(x, y) {
                let i = idx(x as usize, y as usize);
                river[i] |= land[i];
                land[i] = false;
            }
        }
    }
}

fn add_beaches(land: &[bool], seed: u64) -> Vec<Terrain> {
    let mut dist = vec![u8::MAX; MAP_SIZE * MAP_SIZE];
    let mut queue = VecDeque::new();
    for (i, &is_land) in land.iter().enumerate() {
        if !is_land {
            dist[i] = 0;
            queue.push_back(i);
        }
    }

    while let Some(i) = queue.pop_front() {
        if dist[i] >= MAX_BEACH {
            continue;
        }
        let (x, y) = ((i % MAP_SIZE) as i32, (i / MAP_SIZE) as i32);
        for (dx, dy) in NEIGHBORS {
            let (nx, ny) = (x + dx, y + dy);
            if !Map::in_bounds(nx, ny) {
                continue;
            }
            let n = idx(nx as usize, ny as usize);
            if dist[n] == u8::MAX {
                dist[n] = dist[i] + 1;
                queue.push_back(n);
            }
        }
    }

    (0..MAP_SIZE * MAP_SIZE)
        .map(|i| {
            if !land[i] {
                return Terrain::Water;
            }
            let (x, y) = (i % MAP_SIZE, i / MAP_SIZE);
            if dist[i] <= beach_width(x, y, seed) {
                Terrain::Sand
            } else {
                Terrain::Grass
            }
        })
        .collect()
}

fn beach_width(x: usize, y: usize, seed: u64) -> u8 {
    let noise = fbm(x as f32 * 0.08, y as f32 * 0.08, seed ^ 0xBEAC, 2);
    1 + ((noise - 0.35) / 0.3 * 3.0).clamp(0.0, 2.99) as u8
}

fn place_features(tiles: &[Terrain], seed: u64) -> Vec<Option<Feature>> {
    let mut rng = Rng::new(seed ^ FEATURE_SEED);
    let mut features = vec![None; MAP_SIZE * MAP_SIZE];
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let i = idx(x, y);
            if tiles[i] != Terrain::Grass {
                continue;
            }
            let density = fbm(
                x as f32 * FOREST_SCALE,
                y as f32 * FOREST_SCALE,
                seed ^ FEATURE_SEED,
                3,
            );
            let tree_chance = if density >= FOREST_THRESHOLD - FOREST_FRINGE {
                FRINGE_TREE_CHANCE
            } else {
                TREE_CHANCE
            };
            if density >= FOREST_THRESHOLD {
                features[i] = Some(Feature::Forest);
            } else if rng.chance(tree_chance) {
                features[i] = Some(Feature::Tree);
            }
        }
    }

    thin_forests(&mut features);

    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            if !rng.chance(ROCK_CHANCE) {
                continue;
            }
            place_rock(tiles, &mut features, idx(x, y));
            for (dx, dy) in NEIGHBORS {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if Map::in_bounds(nx, ny) && rng.chance(ROCK_SPREAD_CHANCE) {
                    place_rock(tiles, &mut features, idx(nx as usize, ny as usize));
                }
            }
        }
    }
    features
}

// One-tile-thick forest strips look skinny once their edges round off, so they become trees.
fn thin_forests(features: &mut [Option<Feature>]) {
    let is_forest = |features: &[Option<Feature>], x: i32, y: i32| {
        !Map::in_bounds(x, y) || features[idx(x as usize, y as usize)] == Some(Feature::Forest)
    };
    loop {
        let mut changed = false;
        for y in 0..MAP_SIZE as i32 {
            for x in 0..MAP_SIZE as i32 {
                if !is_forest(features, x, y) {
                    continue;
                }
                let vertical = is_forest(features, x, y - 1) || is_forest(features, x, y + 1);
                let horizontal = is_forest(features, x - 1, y) || is_forest(features, x + 1, y);
                if !vertical || !horizontal {
                    features[idx(x as usize, y as usize)] = Some(Feature::Tree);
                    changed = true;
                }
            }
        }
        if !changed {
            return;
        }
    }
}

fn place_rock(tiles: &[Terrain], features: &mut [Option<Feature>], i: usize) {
    if tiles[i] != Terrain::Water && features[i] != Some(Feature::Forest) {
        features[i] = Some(Feature::Rock);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEEDS: [u64; 5] = [1, 7, 42, 1234, 99_999];

    // A 5x5 patch of dry grass with nothing on it, away from the borders.
    fn clear_ground() -> Map {
        let mut map = Map::generate(TerrainType::Inland, 42);
        for y in 30..40 {
            for x in 30..40 {
                let i = idx(x, y);
                map.tiles[i] = Terrain::Grass;
                map.features[i] = None;
                map.roads[i] = None;
                map.power_lines[i] = None;
            }
        }
        map
    }

    #[test]
    fn coal_plant_fills_its_footprint_around_the_center() {
        let mut map = clear_ground();
        assert!(map.build_building(BuildingKind::CoalPlant, (34, 34)));
        let plant = Building { kind: BuildingKind::CoalPlant, x: 32, y: 32 };
        assert_eq!(map.buildings(), vec![plant]);
        for (x, y) in plant.tiles() {
            assert_eq!(map.building_at(x as usize, y as usize), Some(plant));
        }
        assert_eq!(map.building_at(31, 32), None);
        assert_eq!(map.building_at(37, 32), None);
    }

    #[test]
    fn buildings_cannot_overlap_or_sit_on_water_rocks_roads_or_lines() {
        let mut map = clear_ground();
        assert!(map.build_building(BuildingKind::CoalPlant, (34, 34)));
        assert!(!map.build_building(BuildingKind::CoalPlant, (36, 34)));
        assert!(map.build_building(BuildingKind::CoalPlant, (39, 34)));

        let kind = BuildingKind::CoalPlant;
        let mut blocked = clear_ground();
        blocked.tiles[idx(33, 33)] = Terrain::Water;
        assert!(!blocked.can_build_building(kind, (34, 34)));
        let mut blocked = clear_ground();
        blocked.features[idx(35, 36)] = Some(Feature::Rock);
        assert!(!blocked.can_build_building(kind, (34, 34)));
        let mut blocked = clear_ground();
        assert!(blocked.build_road(36, 34));
        assert!(!blocked.can_build_building(kind, (34, 34)));
        let mut blocked = clear_ground();
        assert!(blocked.build_power_line(32, 36));
        assert!(!blocked.can_build_building(kind, (34, 34)));
    }

    #[test]
    fn buildings_must_fit_inside_the_map() {
        let map = Map::generate(TerrainType::Inland, 42);
        assert!(!map.can_build_building(BuildingKind::CoalPlant, (0, 0)));
        assert!(!map.can_build_building(BuildingKind::CoalPlant, (MAP_SIZE as i32 - 1, 100)));
    }

    #[test]
    fn building_clears_trees_and_forests() {
        let mut map = clear_ground();
        map.features[idx(34, 34)] = Some(Feature::Forest);
        map.features[idx(32, 32)] = Some(Feature::Tree);
        assert!(map.build_building(BuildingKind::CoalPlant, (34, 34)));
        assert_eq!(map.feature(34, 34), None);
        assert_eq!(map.feature(32, 32), None);
    }

    #[test]
    fn roads_and_power_lines_cannot_be_built_on_a_building() {
        let mut map = clear_ground();
        assert!(map.build_building(BuildingKind::CoalPlant, (34, 34)));
        assert!(!map.can_build_road(34, 34));
        assert!(!map.can_build_power_line(32, 36));
        assert!(map.can_build_road(31, 34));
    }

    #[test]
    fn bulldozing_any_tile_removes_the_whole_building() {
        let mut map = clear_ground();
        assert!(map.build_building(BuildingKind::CoalPlant, (34, 34)));
        assert!(map.can_bulldoze(36, 36));
        assert!(map.bulldoze(36, 36));
        assert!(map.buildings().is_empty());
        assert!(map.can_build_building(BuildingKind::CoalPlant, (34, 34)));
        assert!(!map.bulldoze(34, 34));
    }

    #[test]
    fn bulldoze_removes_roads_and_features() {
        let mut map = Map::generate(TerrainType::Inland, 42);
        let (x, y) = (10, 10);
        map.features[idx(x, y)] = Some(Feature::Rock);
        assert!(map.bulldoze(x as i32, y as i32));
        assert_eq!(map.feature(x, y), None);
        assert!(!map.bulldoze(x as i32, y as i32));

        map.features[idx(x, y)] = None;
        map.roads[idx(x, y)] = Some(Road::Street);
        assert!(map.bulldoze(x as i32, y as i32));
        assert_eq!(map.road(x, y), None);
        assert!(!map.bulldoze(-1, 0));
    }

    fn blank_land(x: usize, y: usize, terrain: Terrain) -> Map {
        let mut map = Map::generate(TerrainType::Inland, 42);
        map.features = vec![None; MAP_SIZE * MAP_SIZE];
        map.roads = vec![None; MAP_SIZE * MAP_SIZE];
        for dy in 0..3 {
            for dx in 0..4 {
                map.tiles[idx(x + dx, y + dy - 1)] = terrain;
            }
        }
        map
    }

    #[test]
    fn power_lines_replace_trees_but_not_rocks() {
        let mut map = blank_land(10, 10, Terrain::Grass);
        map.features[idx(10, 10)] = Some(Feature::Tree);
        map.features[idx(11, 10)] = Some(Feature::Rock);
        assert!(map.build_power_line(10, 10));
        assert_eq!(map.power_line(10, 10), Some(PowerLine::Pole));
        assert!(map.feature(10, 10).is_none());
        assert!(!map.can_build_power_line(10, 10));
        assert!(!map.can_build_power_line(11, 10));
        assert!(!map.can_build_power_line(-1, 0));
    }

    #[test]
    fn roads_and_power_lines_share_a_tile_in_either_order() {
        let mut map = blank_land(10, 10, Terrain::Grass);
        assert!(map.build_road(10, 10));
        assert!(map.build_power_line(10, 10));
        assert!(map.build_power_line(11, 10));
        assert!(map.build_road(11, 10));
        for x in [10, 11] {
            assert_eq!(map.road(x, 10), Some(Road::Street));
            assert_eq!(map.power_line(x, 10), Some(PowerLine::Pole));
        }
    }

    #[test]
    fn power_line_bridges_start_from_a_power_line_and_only_extend_straight() {
        let mut map = blank_land(10, 10, Terrain::Water);
        map.tiles[idx(10, 10)] = Terrain::Grass;
        assert!(!map.can_build_power_line(11, 10));
        map.build_power_line(10, 10);
        assert!(map.build_power_line(11, 10));
        assert_eq!(map.power_line(11, 10), Some(PowerLine::Bridge(Axis::Horizontal)));
        assert!(!map.can_build_power_line(11, 11));
        assert!(map.can_build_power_line(12, 10));
    }

    #[test]
    fn power_lines_and_road_bridges_exclude_each_other_over_water() {
        let mut map = blank_land(10, 10, Terrain::Water);
        map.tiles[idx(10, 10)] = Terrain::Grass;
        map.build_road(10, 10);
        map.build_power_line(10, 10);
        assert!(map.build_road(11, 10));
        assert!(!map.can_build_power_line(11, 10));
        map.bulldoze(11, 10);
        assert!(map.build_power_line(11, 10));
        assert!(!map.can_build_road(11, 10));
    }

    #[test]
    fn bulldozing_removes_the_top_layer_first() {
        let mut map = blank_land(10, 10, Terrain::Grass);
        map.build_road(10, 10);
        map.build_power_line(10, 10);
        assert!(map.bulldoze(10, 10));
        assert_eq!((map.road(10, 10), map.power_line(10, 10)), (Some(Road::Street), None));
        assert!(map.bulldoze(10, 10));
        assert_eq!(map.road(10, 10), None);
        assert!(!map.bulldoze(10, 10));
    }

    fn land_ratio(map: &Map) -> f32 {
        let land = map.tiles.iter().filter(|&&t| t != Terrain::Water).count();
        land as f32 / (MAP_SIZE * MAP_SIZE) as f32
    }

    fn edge_tiles(map: &Map, side: Side) -> Vec<Terrain> {
        let last = MAP_SIZE - 1;
        (0..MAP_SIZE)
            .map(|i| match side {
                Side::North => map.terrain(i, 0),
                Side::South => map.terrain(i, last),
                Side::West => map.terrain(0, i),
                Side::East => map.terrain(last, i),
            })
            .collect()
    }

    fn has_land(tiles: &[Terrain]) -> bool {
        tiles.iter().any(|&t| t != Terrain::Water)
    }

    fn all_water(tiles: &[Terrain]) -> bool {
        tiles.iter().all(|&t| t == Terrain::Water)
    }

    fn fingerprint(map: &Map) -> u64 {
        let cells = map.tiles.iter().zip(&map.river);
        cells.fold(0xCBF2_9CE4_8422_2325, |hash, (&terrain, &river)| {
            let byte = terrain as u64 | (river as u64) << 2;
            (hash ^ byte).wrapping_mul(0x0100_0000_01B3)
        })
    }

    #[test]
    fn generation_output_is_stable_across_refactors() {
        let expected = [
            0x47b5_b26c_c0e3_4ea3,
            0x3e14_99eb_8a7b_45ed,
            0xf506_3549_621e_81d3,
            0x8532_ebde_a11f_923c,
        ];
        for (kind, expected) in TerrainType::ALL.into_iter().zip(expected) {
            assert_eq!(fingerprint(&Map::generate(kind, 42)), expected, "{kind:?}");
        }
    }

    #[test]
    fn generation_is_deterministic_per_seed() {
        for kind in TerrainType::ALL {
            let a = Map::generate(kind, 5);
            let b = Map::generate(kind, 5);
            assert!(a.tiles == b.tiles);
        }
    }

    #[test]
    fn different_seeds_give_different_maps() {
        for kind in TerrainType::ALL {
            assert!(Map::generate(kind, 1).tiles != Map::generate(kind, 2).tiles);
        }
    }

    #[test]
    fn inland_is_mostly_land_and_touches_all_edges() {
        for seed in SEEDS {
            let map = Map::generate(TerrainType::Inland, seed);
            assert!(land_ratio(&map) > 0.9);
            for side in SIDES {
                assert!(has_land(&edge_tiles(&map, side)));
            }
        }
    }

    #[test]
    fn inland_river_crosses_the_map() {
        for seed in SEEDS {
            let map = Map::generate(TerrainType::Inland, seed);
            let north = edge_tiles(&map, Side::North).contains(&Terrain::Water);
            let south = edge_tiles(&map, Side::South).contains(&Terrain::Water);
            let west = edge_tiles(&map, Side::West).contains(&Terrain::Water);
            let east = edge_tiles(&map, Side::East).contains(&Terrain::Water);
            assert!((north && south) || (west && east));
        }
    }

    #[test]
    fn island_is_mostly_land_surrounded_by_water() {
        for seed in SEEDS {
            let map = Map::generate(TerrainType::Island, seed);
            assert!(land_ratio(&map) > 0.5);
            for side in SIDES {
                assert!(all_water(&edge_tiles(&map, side)));
            }
        }
    }

    #[test]
    fn archipelago_is_half_land_with_central_mainland() {
        for seed in SEEDS {
            let map = Map::generate(TerrainType::Archipelago, seed);
            assert!((land_ratio(&map) - 0.5).abs() < 0.08);
            assert_ne!(map.terrain(MAP_SIZE / 2, MAP_SIZE / 2), Terrain::Water);
            for side in SIDES {
                assert!(all_water(&edge_tiles(&map, side)));
            }
        }
    }

    #[test]
    fn seaside_has_exactly_one_full_water_edge() {
        for seed in SEEDS {
            let map = Map::generate(TerrainType::Seaside, seed);
            assert!((1.0 - land_ratio(&map) - SEA_RATIO).abs() < 0.05);
            let sea_edges = SIDES
                .iter()
                .filter(|&&side| all_water(&edge_tiles(&map, side)))
                .count();
            assert_eq!(sea_edges, 1);
        }
    }

    #[test]
    fn seaside_river_spans_from_the_far_edge_to_the_sea() {
        for seed in SEEDS {
            let map = Map::generate(TerrainType::Seaside, seed);
            let has_river = (0..MAP_SIZE * MAP_SIZE).any(|i| map.river[i]);
            if !has_river {
                continue;
            }
            let reaches = |x: usize, y: usize| map.is_river(x, y);
            let far_edge = (0..MAP_SIZE).any(|i| {
                reaches(i, 0) || reaches(i, MAP_SIZE - 1) || reaches(0, i) || reaches(MAP_SIZE - 1, i)
            });
            assert!(far_edge);
        }
    }

    fn count(map: &Map, feature: Feature) -> usize {
        map.features.iter().filter(|&&f| f == Some(feature)).count()
    }

    #[test]
    fn features_are_deterministic_per_seed() {
        for kind in TerrainType::ALL {
            assert!(Map::generate(kind, 5).features == Map::generate(kind, 5).features);
            assert!(Map::generate(kind, 1).features != Map::generate(kind, 2).features);
        }
    }

    #[test]
    fn features_sit_only_on_allowed_terrain() {
        for kind in TerrainType::ALL {
            for seed in SEEDS {
                let map = Map::generate(kind, seed);
                for y in 0..MAP_SIZE {
                    for x in 0..MAP_SIZE {
                        let terrain = map.terrain(x, y);
                        match map.feature(x, y) {
                            Some(Feature::Tree | Feature::Forest) => {
                                assert_eq!(terrain, Terrain::Grass)
                            }
                            Some(Feature::Rock) => assert_ne!(terrain, Terrain::Water),
                            None => {}
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn every_map_has_trees_rocks_and_some_forest() {
        for kind in TerrainType::ALL {
            for seed in SEEDS {
                let map = Map::generate(kind, seed);
                let grass = map.tiles.iter().filter(|&&t| t == Terrain::Grass).count();
                let forest = count(&map, Feature::Forest) as f32 / grass as f32;
                assert!(count(&map, Feature::Tree) > 0, "{kind:?} {seed}");
                assert!(count(&map, Feature::Rock) > 0, "{kind:?} {seed}");
                assert!((0.02..0.3).contains(&forest), "{kind:?} {seed} {forest}");
            }
        }
    }

    #[test]
    fn forests_are_clustered() {
        for seed in SEEDS {
            let map = Map::generate(TerrainType::Inland, seed);
            let mut clustered = 0;
            for y in 0..MAP_SIZE {
                for x in 0..MAP_SIZE {
                    if map.feature(x, y) != Some(Feature::Forest) {
                        continue;
                    }
                    let neighbors = NEIGHBORS
                        .iter()
                        .filter(|&&(dx, dy)| {
                            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                            Map::in_bounds(nx, ny)
                                && map.feature(nx as usize, ny as usize) == Some(Feature::Forest)
                        })
                        .count();
                    if neighbors >= 2 {
                        clustered += 1;
                    }
                }
            }
            assert!(clustered as f32 > count(&map, Feature::Forest) as f32 * 0.9);
        }
    }

    #[test]
    fn forests_are_never_one_tile_thick() {
        for kind in TerrainType::ALL {
            let map = Map::generate(kind, 9);
            let forest = |x: i32, y: i32| {
                !Map::in_bounds(x, y) || map.feature(x as usize, y as usize) == Some(Feature::Forest)
            };
            for y in 0..MAP_SIZE as i32 {
                for x in 0..MAP_SIZE as i32 {
                    if map.feature(x as usize, y as usize) == Some(Feature::Forest) {
                        assert!(forest(x, y - 1) || forest(x, y + 1));
                        assert!(forest(x - 1, y) || forest(x + 1, y));
                    }
                }
            }
        }
    }

    #[test]
    fn grass_never_touches_water() {
        for kind in TerrainType::ALL {
            for seed in SEEDS {
                let map = Map::generate(kind, seed);
                for y in 0..MAP_SIZE {
                    for x in 0..MAP_SIZE {
                        if map.terrain(x, y) != Terrain::Grass {
                            continue;
                        }
                        for (dx, dy) in NEIGHBORS {
                            let n = map.get_or(x as i32 + dx, y as i32 + dy, Terrain::Grass);
                            assert_ne!(n, Terrain::Water);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn sand_is_at_most_three_tiles_from_water() {
        for kind in TerrainType::ALL {
            let map = Map::generate(kind, 3);
            for y in 0..MAP_SIZE as i32 {
                for x in 0..MAP_SIZE as i32 {
                    if map.terrain(x as usize, y as usize) != Terrain::Sand {
                        continue;
                    }
                    let near_water = (-3..=3).any(|dy| {
                        (-3..=3).any(|dx| map.get_or(x + dx, y + dy, Terrain::Grass) == Terrain::Water)
                    });
                    assert!(near_water);
                }
            }
        }
    }
}
