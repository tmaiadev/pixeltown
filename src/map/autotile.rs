use super::{Axis, Feature, Map, PowerLine, Road, Terrain, MAP_SIZE};

pub const SHEET_COLUMNS: usize = 16;
const GRASS_VARIANTS: usize = 4;
const SAND_VARIANTS: usize = 3;
const WATER_VARIANTS: usize = 3;
const SAND_START: usize = GRASS_VARIANTS;
const WATER_START: usize = SAND_START + SAND_VARIANTS;
const SHORE_START: usize = SHEET_COLUMNS;
const GRASS_EDGE_START: usize = 4 * SHEET_COLUMNS;

const FEATURE_VARIANTS: usize = 3;
const TREE_START: usize = 0;
const ROCK_START: usize = TREE_START + FEATURE_VARIANTS;
const TREE_TRUNK: usize = ROCK_START + FEATURE_VARIANTS;
const TREE_SHADOW: usize = TREE_TRUNK + 1;
const FOREST_START: usize = SHEET_COLUMNS;
const FOREST_PERIOD: usize = 2;

pub const WATER_FRAMES: usize = 4;
const FRAMES_START_ROW: usize = 7;
const FRAME_ROWS: usize = 4;

const N: u8 = 1;
const E: u8 = 2;
const S: u8 = 4;
const W: u8 = 8;
const NE: u8 = 16;
const SE: u8 = 32;
const SW: u8 = 64;
const NW: u8 = 128;

const CARDINALS: [(i32, i32, u8); 4] = [(0, -1, N), (1, 0, E), (0, 1, S), (-1, 0, W)];
const DIAGONALS: [(i32, i32, u8); 4] = [(1, -1, NE), (1, 1, SE), (-1, 1, SW), (-1, -1, NW)];

const fn normalize(mask: u8) -> u8 {
    let mut m = mask & 0x0F;
    if mask & NE != 0 && mask & (N | E) == 0 {
        m |= NE;
    }
    if mask & SE != 0 && mask & (S | E) == 0 {
        m |= SE;
    }
    if mask & SW != 0 && mask & (S | W) == 0 {
        m |= SW;
    }
    if mask & NW != 0 && mask & (N | W) == 0 {
        m |= NW;
    }
    m
}

const BLOB_COUNT: usize = 47;

const BLOB_INDEX: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut next = 0;
    let mut m = 0;
    while m < 256 {
        if normalize(m as u8) == m as u8 {
            table[m] = next;
            next += 1;
        }
        m += 1;
    }
    m = 0;
    while m < 256 {
        table[m] = table[normalize(m as u8) as usize];
        m += 1;
    }
    table
};

const _: () = assert!(SHORE_START + BLOB_COUNT <= GRASS_EDGE_START);

fn mask_where(x: usize, y: usize, matches: impl Fn(i32, i32) -> bool) -> u8 {
    let (x, y) = (x as i32, y as i32);
    let mut mask = 0;
    for (dx, dy, bit) in CARDINALS.into_iter().chain(DIAGONALS) {
        if matches(x + dx, y + dy) {
            mask |= bit;
        }
    }
    normalize(mask)
}

fn neighbor_mask(map: &Map, x: usize, y: usize, other: Terrain) -> u8 {
    let own = map.terrain(x, y);
    mask_where(x, y, |nx, ny| map.get_or(nx, ny, own) == other)
}

// Map edges count as forest so forests run off the map instead of closing up.
fn forest_edge_mask(map: &Map, x: usize, y: usize) -> u8 {
    mask_where(x, y, |nx, ny| {
        Map::in_bounds(nx, ny) && map.feature(nx as usize, ny as usize) != Some(Feature::Forest)
    })
}

fn variant(x: usize, y: usize, count: usize) -> usize {
    let h = (x as u32).wrapping_mul(73_856_093) ^ (y as u32).wrapping_mul(19_349_663);
    ((h >> 8) as usize) % count
}

// Column and row of a sprite in a spritesheet.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpriteCell {
    col: u8,
    row: u8,
}

impl SpriteCell {
    pub const fn new(col: usize, row: usize) -> Self {
        Self {
            col: col as u8,
            row: row as u8,
        }
    }

    pub fn col(self) -> usize {
        self.col as usize
    }

    pub fn row(self) -> usize {
        self.row as usize
    }
}

fn cell(index: usize) -> SpriteCell {
    SpriteCell::new(index % SHEET_COLUMNS, index / SHEET_COLUMNS)
}

// Every map position in row-major order.
fn tiles() -> impl Iterator<Item = (usize, usize)> {
    (0..MAP_SIZE).flat_map(|y| (0..MAP_SIZE).map(move |x| (x, y)))
}

fn per_tile<T>(f: impl Fn(usize, usize) -> T) -> Vec<T> {
    tiles().map(|(x, y)| f(x, y)).collect()
}

pub fn tile_cell(map: &Map, x: usize, y: usize) -> SpriteCell {
    match map.terrain(x, y) {
        Terrain::Water => cell(WATER_START + variant(x, y, WATER_VARIANTS)),
        Terrain::Sand => match neighbor_mask(map, x, y, Terrain::Water) {
            0 => cell(SAND_START + variant(x, y, SAND_VARIANTS)),
            mask => cell(SHORE_START + BLOB_INDEX[mask as usize] as usize),
        },
        Terrain::Grass => match neighbor_mask(map, x, y, Terrain::Sand) {
            0 => cell(variant(x, y, GRASS_VARIANTS)),
            mask => cell(GRASS_EDGE_START + BLOB_INDEX[mask as usize] as usize),
        },
    }
}

pub fn feature_cell(map: &Map, x: usize, y: usize) -> Option<SpriteCell> {
    let index = match map.feature(x, y)? {
        Feature::Rock => ROCK_START + variant(x, y, FEATURE_VARIANTS),
        // Trees and forests are drawn in perspective, not baked with the ground.
        Feature::Tree | Feature::Forest => return None,
    };
    Some(cell(index))
}

const BRIDGE_DECK_ROW: usize = 1;
const BRIDGE_TRUSS_ROW: usize = 2;
const CROSSWALK_ROW: usize = 3;
const DEAD_END_CROSSWALKS: usize = 6;

fn bridge_col(axis: Axis) -> usize {
    match axis {
        Axis::Vertical => 0,
        Axis::Horizontal => 1,
    }
}

// Whether the road at (x, y) links to its neighbor in direction (dx, dy); bridges only link along their axis.
fn linked(map: &Map, x: usize, y: usize, dx: i32, dy: i32) -> bool {
    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
    if !Map::in_bounds(nx, ny) {
        return false;
    }
    let axis = if dx == 0 { Axis::Vertical } else { Axis::Horizontal };
    let straight = |road| !matches!(road, Some(Road::Bridge(a)) if a != axis);
    map.road(nx as usize, ny as usize).is_some() && straight(map.road(x, y)) && straight(map.road(nx as usize, ny as usize))
}

fn street_mask(map: &Map, x: usize, y: usize) -> u8 {
    CARDINALS
        .into_iter()
        .filter(|&(dx, dy, _)| linked(map, x, y, dx, dy))
        .fold(0u8, |mask, (_, _, bit)| mask | bit)
}

fn is_junction(map: &Map, x: i32, y: i32) -> bool {
    Map::in_bounds(x, y)
        && map.road(x as usize, y as usize) == Some(Road::Street)
        && street_mask(map, x as usize, y as usize).count_ones() >= 3
}

// Streets get a pedestrian crossing on each end that meets a junction; dead ends only have the one end.
fn crosswalk_col(map: &Map, x: usize, y: usize, mask: u8) -> Option<usize> {
    let (first, second, start) = match mask {
        m if m == N | S || m == N || m == S => (N, S, 0),
        m if m == W | E || m == W || m == E => (W, E, 3),
        _ => return None,
    };
    let at = |bit| {
        let (dx, dy, _) = CARDINALS.into_iter().find(|&(_, _, b)| b == bit).unwrap();
        mask & bit != 0 && is_junction(map, x as i32 + dx, y as i32 + dy)
    };
    match (at(first), at(second)) {
        (false, false) => None,
        (near, far) if mask.count_ones() == 2 => Some(start + near as usize + 2 * far as usize - 1),
        _ => Some(DEAD_END_CROSSWALKS + [N, S, W, E].iter().position(|&b| b == mask).unwrap()),
    }
}

// Street sprites are indexed by the N/E/S/W mask; an isolated street is drawn vertical.
pub fn road_cell(map: &Map, x: usize, y: usize) -> Option<SpriteCell> {
    match map.road(x, y)? {
        Road::Bridge(axis) => Some(SpriteCell::new(bridge_col(axis), BRIDGE_DECK_ROW)),
        Road::Street => {
            let mask = street_mask(map, x, y);
            if let Some(col) = crosswalk_col(map, x, y, mask) {
                return Some(SpriteCell::new(col, CROSSWALK_ROW));
            }
            Some(SpriteCell::new(if mask == 0 { N | S } else { mask } as usize, 0))
        }
    }
}

// The metal structure drawn raised above a bridge.
pub fn bridge_truss_cell(map: &Map, x: usize, y: usize) -> Option<SpriteCell> {
    match map.road(x, y)? {
        Road::Bridge(axis) => Some(SpriteCell::new(bridge_col(axis), BRIDGE_TRUSS_ROW)),
        Road::Street => None,
    }
}

pub fn bridge_truss_cells(map: &Map) -> Vec<Option<SpriteCell>> {
    per_tile(|x, y| bridge_truss_cell(map, x, y))
}

pub fn road_cells(map: &Map) -> Vec<Option<SpriteCell>> {
    per_tile(|x, y| road_cell(map, x, y))
}

// How a power line is held up: a pole in the middle of its tile, or a wire spanning the tile
// along an axis (over water, or over a street, which it crosses).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wiring {
    Pole,
    Span(Axis),
}

fn perpendicular(axis: Axis) -> Axis {
    match axis {
        Axis::Horizontal => Axis::Vertical,
        Axis::Vertical => Axis::Horizontal,
    }
}

// Streets run horizontally only when they link sideways and not up or down; the rest count as vertical, like their sprite.
fn road_axis(map: &Map, x: usize, y: usize) -> Axis {
    let linked = |dx, dy| linked(map, x, y, dx, dy);
    let horizontal = linked(1, 0) || linked(-1, 0);
    let vertical = linked(0, -1) || linked(0, 1);
    if horizontal && !vertical {
        Axis::Horizontal
    } else {
        Axis::Vertical
    }
}

// Whatever sits over a street takes no tile space, so it crosses the street instead of following it.
pub fn wiring(map: &Map, x: usize, y: usize) -> Option<Wiring> {
    Some(match map.power_line(x, y)? {
        PowerLine::Bridge(axis) => Wiring::Span(axis),
        PowerLine::Pole if map.road(x, y).is_some() => Wiring::Span(perpendicular(road_axis(map, x, y))),
        PowerLine::Pole => Wiring::Pole,
    })
}

// The wiring of the neighbor in direction (dx, dy) when the wire at (x, y) reaches it.
fn wired(map: &Map, x: usize, y: usize, dx: i32, dy: i32) -> Option<Wiring> {
    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
    if !Map::in_bounds(nx, ny) {
        return None;
    }
    let axis = if dx == 0 { Axis::Vertical } else { Axis::Horizontal };
    let reaches = |wiring| !matches!(wiring, Wiring::Span(a) if a != axis);
    let (here, there) = (wiring(map, x, y)?, wiring(map, nx as usize, ny as usize)?);
    (reaches(here) && reaches(there)).then_some(there)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WireCell {
    pub x: u16,
    pub y: u16,
    pub wiring: Wiring,
    // Wiring of the neighbor east and south of it, when linked. Each wire is owned by one of its ends, so it is drawn once.
    pub east: Option<Wiring>,
    pub south: Option<Wiring>,
    // N/E/S/W bits of the sides a spanning wire runs out of the tile without a neighbor to link to.
    pub open: u8,
}

// Row-major.
pub fn wire_cells(map: &Map) -> Vec<WireCell> {
    tiles()
        .filter_map(|(x, y)| {
            let wiring = wiring(map, x, y)?;
            let open = match wiring {
                Wiring::Pole => 0,
                Wiring::Span(axis) => {
                    let ends = match axis {
                        Axis::Vertical => [(0, -1, N), (0, 1, S)],
                        Axis::Horizontal => [(1, 0, E), (-1, 0, W)],
                    };
                    ends.into_iter()
                        .filter(|&(dx, dy, _)| wired(map, x, y, dx, dy).is_none())
                        .fold(0, |open, (_, _, bit)| open | bit)
                }
            };
            Some(WireCell {
                x: x as u16,
                y: y as u16,
                wiring,
                east: wired(map, x, y, 1, 0),
                south: wired(map, x, y, 0, 1),
                open,
            })
        })
        .collect()
}

pub fn forest_cell(map: &Map, x: usize, y: usize) -> Option<SpriteCell> {
    if map.feature(x, y) != Some(Feature::Forest) {
        return None;
    }
    // The canopy texture spans FOREST_PERIOD tiles, so pick the slice by position.
    let blob = BLOB_INDEX[forest_edge_mask(map, x, y) as usize] as usize;
    let position = (y % FOREST_PERIOD) * FOREST_PERIOD + x % FOREST_PERIOD;
    Some(cell(FOREST_START + blob * FOREST_PERIOD * FOREST_PERIOD + position))
}

pub fn forest_cells(map: &Map) -> Vec<Option<SpriteCell>> {
    per_tile(|x, y| forest_cell(map, x, y))
}

// Only edge trunks can peek out from under the canopy, so inner tiles get none.
pub fn forest_trunks(map: &Map) -> Vec<(u16, u16)> {
    tiles()
        .filter(|&(x, y)| map.feature(x, y) == Some(Feature::Forest) && forest_edge_mask(map, x, y) != 0)
        .map(|(x, y)| (x as u16, y as u16))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeCell {
    pub x: u16,
    pub y: u16,
    pub crown: SpriteCell,
}

pub fn tree_trunk() -> SpriteCell {
    cell(TREE_TRUNK)
}

pub fn tree_shadow() -> SpriteCell {
    cell(TREE_SHADOW)
}

// Row-major, so drawing in order paints lower trees over upper ones.
pub fn tree_cells(map: &Map) -> Vec<TreeCell> {
    tiles()
        .filter(|&(x, y)| map.feature(x, y) == Some(Feature::Tree))
        .map(|(x, y)| TreeCell {
            x: x as u16,
            y: y as u16,
            crown: cell(TREE_START + variant(x, y, FEATURE_VARIANTS)),
        })
        .collect()
}

pub fn feature_cells(map: &Map) -> Vec<Option<SpriteCell>> {
    per_tile(|x, y| feature_cell(map, x, y))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GroundSprite {
    cell: SpriteCell,
    animated: bool,
}

impl GroundSprite {
    fn new(cell: SpriteCell, animated: bool) -> Self {
        Self { cell, animated }
    }

    pub fn is_animated(self) -> bool {
        self.animated
    }
}

fn is_animated(map: &Map, x: usize, y: usize) -> bool {
    match map.terrain(x, y) {
        Terrain::Water => true,
        Terrain::Sand => {
            let (x, y) = (x as i32, y as i32);
            CARDINALS.into_iter().chain(DIAGONALS).any(|(dx, dy, _)| {
                let (nx, ny) = (x + dx, y + dy);
                Map::in_bounds(nx, ny)
                    && map.terrain(nx as usize, ny as usize) == Terrain::Water
                    && !map.is_river(nx as usize, ny as usize)
            })
        }
        Terrain::Grass => false,
    }
}

// `phase` desyncs open water between tiles; shores always follow the global frame.
pub fn animated_cell(sprite: GroundSprite, frame: usize, phase: usize) -> SpriteCell {
    let (col, row) = (sprite.cell.col(), sprite.cell.row());
    let open_water = row == 0 && (WATER_START..WATER_START + WATER_VARIANTS).contains(&col);
    let shore = (1..FRAME_ROWS).contains(&row);
    let frame = match (sprite.animated, open_water, shore) {
        (true, true, _) => (frame + phase) % WATER_FRAMES,
        (true, _, true) => frame % WATER_FRAMES,
        _ => 0,
    };
    let row = match frame {
        0 => row,
        f => FRAMES_START_ROW + (f - 1) * FRAME_ROWS + row,
    };
    SpriteCell::new(col, row)
}

pub fn ground_sprites(map: &Map) -> Vec<GroundSprite> {
    per_tile(|x, y| GroundSprite::new(tile_cell(map, x, y), is_animated(map, x, y)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{idx, TerrainType};

    fn map_from(rows: &[&str]) -> Map {
        let mut tiles = vec![Terrain::Grass; MAP_SIZE * MAP_SIZE];
        let mut river = vec![false; MAP_SIZE * MAP_SIZE];
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                river[idx(x, y)] = c == 'r';
                tiles[idx(x, y)] = match c {
                    '~' | 'r' => Terrain::Water,
                    '.' => Terrain::Sand,
                    _ => Terrain::Grass,
                };
            }
        }
        let features = vec![None; MAP_SIZE * MAP_SIZE];
        Map {
            terrain_type: TerrainType::Inland,
            seed: 0,
            tiles,
            river,
            features,
            roads: vec![None; MAP_SIZE * MAP_SIZE],
            power_lines: vec![None; MAP_SIZE * MAP_SIZE],
        }
    }

    fn cell_at(map: &Map, x: u16, y: u16) -> WireCell {
        wire_cells(map).into_iter().find(|c| (c.x, c.y) == (x, y)).unwrap()
    }

    #[test]
    fn poles_link_to_the_poles_east_and_south() {
        let mut map = map_from(&["###", "###"]);
        map.build_power_line(0, 0);
        let alone = WireCell { x: 0, y: 0, wiring: Wiring::Pole, east: None, south: None, open: 0 };
        assert_eq!(wire_cells(&map), vec![alone]);
        map.build_power_line(1, 0);
        map.build_power_line(0, 1);
        let cells = wire_cells(&map);
        assert_eq!(cells.len(), 3);
        assert_eq!((cells[0].east, cells[0].south), (Some(Wiring::Pole), Some(Wiring::Pole)));
        assert_eq!((cells[1].east, cells[1].south), (None, None));
    }

    #[test]
    fn power_lines_cross_streets_at_a_right_angle() {
        let mut map = map_from(&["###", "###", "###"]);
        for x in 0..3 {
            map.build_road(x, 1);
        }
        map.build_power_line(1, 0);
        map.build_power_line(1, 1);
        map.build_power_line(1, 2);
        assert_eq!(cell_at(&map, 1, 1).wiring, Wiring::Span(Axis::Vertical));
        assert_eq!(cell_at(&map, 1, 1).south, Some(Wiring::Pole));
        assert_eq!(cell_at(&map, 1, 0).south, Some(Wiring::Span(Axis::Vertical)));

        let mut map = map_from(&["###", "###", "###"]);
        for y in 0..3 {
            map.build_road(1, y);
        }
        map.build_power_line(1, 1);
        assert_eq!(cell_at(&map, 1, 1).wiring, Wiring::Span(Axis::Horizontal));
        assert_eq!(cell_at(&map, 1, 1).open, E | W);
    }

    #[test]
    fn power_lines_over_water_have_no_poles_and_link_both_ends() {
        let mut map = map_from(&["#~~#"]);
        map.build_power_line(0, 0);
        map.build_power_line(1, 0);
        map.build_power_line(2, 0);
        map.build_power_line(3, 0);
        let cells = wire_cells(&map);
        assert_eq!(cells[1].wiring, Wiring::Span(Axis::Horizontal));
        assert_eq!(cells[1].open, 0);
        assert_eq!((cells[0].east, cells[3].wiring), (Some(Wiring::Span(Axis::Horizontal)), Wiring::Pole));
    }

    #[test]
    fn features_pick_cells_from_their_own_section() {
        let mut map = map_from(&["###"]);
        assert_eq!(feature_cell(&map, 0, 0), None);
        for x in 0..3 {
            map.features[idx(x, 0)] = Some(Feature::Rock);
            let cell = feature_cell(&map, x, 0).unwrap();
            assert_eq!(cell.row(), 0);
            assert!((ROCK_START..ROCK_START + FEATURE_VARIANTS).contains(&cell.col()));
        }
    }

    #[test]
    fn trees_and_forests_are_not_baked_with_the_ground() {
        let mut map = map_from(&["##"]);
        map.features[idx(0, 0)] = Some(Feature::Tree);
        map.features[idx(1, 0)] = Some(Feature::Forest);
        assert_eq!(feature_cell(&map, 0, 0), None);
        assert_eq!(feature_cell(&map, 1, 0), None);
        assert_eq!(forest_cell(&map, 0, 0), None);
        assert!(forest_cell(&map, 1, 0).is_some());
    }

    #[test]
    fn tree_cells_list_every_tree_in_row_major_order() {
        let mut map = map_from(&["###", "###"]);
        for (x, y) in [(2, 0), (0, 1), (1, 1)] {
            map.features[idx(x, y)] = Some(Feature::Tree);
        }
        map.features[idx(0, 0)] = Some(Feature::Rock);
        let trees = tree_cells(&map);
        let positions: Vec<_> = trees.iter().map(|t| (t.x, t.y)).collect();
        assert_eq!(positions, [(2, 0), (0, 1), (1, 1)]);
        for tree in trees {
            assert_eq!(tree.crown.row(), 0);
            assert!((TREE_START..TREE_START + FEATURE_VARIANTS).contains(&tree.crown.col()));
        }
    }

    fn index(cell: SpriteCell) -> usize {
        cell.row() * SHEET_COLUMNS + cell.col()
    }

    fn forest_index(map: &Map, x: usize, y: usize) -> usize {
        index(forest_cell(map, x, y).unwrap()) - FOREST_START
    }

    #[test]
    fn forest_inside_uses_plain_canopy_sliced_by_position() {
        let mut map = map_from(&["###", "###", "###"]);
        for i in 0..9 {
            map.features[idx(i % 3, i / 3)] = Some(Feature::Forest);
        }
        assert_eq!(forest_index(&map, 1, 1), 3);
        assert_eq!(forest_index(&map, 0, 0), 0);
        assert_eq!(forest_index(&map, 1, 0), 1);
        assert_eq!(forest_index(&map, 0, 1), 2);
    }

    #[test]
    fn forest_edge_picks_blob_for_non_forest_neighbors() {
        let mut map = map_from(&["###", "###", "###"]);
        for i in 3..9 {
            map.features[idx(i % 3, i / 3)] = Some(Feature::Forest);
        }
        let blob = BLOB_INDEX[N as usize] as usize;
        assert_eq!(forest_index(&map, 1, 1), blob * 4 + 3);
    }

    #[test]
    fn forest_trunks_sit_only_on_edge_tiles() {
        let mut map = map_from(&["#####", "#####", "#####", "#####", "#####"]);
        for y in 1..4 {
            for x in 1..4 {
                map.features[idx(x, y)] = Some(Feature::Forest);
            }
        }
        let trunks = forest_trunks(&map);
        assert_eq!(trunks.len(), 8);
        assert!(!trunks.contains(&(2, 2)));
    }

    #[test]
    fn there_are_47_distinct_blob_indices() {
        let mut seen: Vec<u8> = (0..=255u8).map(|m| BLOB_INDEX[m as usize]).collect();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), BLOB_COUNT);
        assert_eq!(*seen.last().unwrap() as usize, BLOB_COUNT - 1);
    }

    #[test]
    fn diagonal_bits_are_dropped_when_a_cardinal_covers_them() {
        assert_eq!(normalize(N | NE), N);
        assert_eq!(normalize(NE), NE);
        assert_eq!(BLOB_INDEX[(N | NE) as usize], BLOB_INDEX[N as usize]);
    }

    #[test]
    fn tiles_without_lower_neighbors_use_base_variants() {
        let flat = map_from(&["...", "...", "..."]);
        let cell = tile_cell(&flat, 1, 1);
        assert_eq!(cell.row(), 0);
        assert!((SAND_START..WATER_START).contains(&cell.col()));
    }

    #[test]
    fn sand_next_to_water_uses_shore_section() {
        let map = map_from(&["~~~", "~..", "~.."]);
        let index = index(tile_cell(&map, 1, 1));
        assert!((SHORE_START..GRASS_EDGE_START).contains(&index));
        assert_eq!(index - SHORE_START, BLOB_INDEX[(N | W) as usize] as usize);
    }

    #[test]
    fn grass_next_to_sand_uses_grass_edge_section() {
        let map = map_from(&[".##", ".##", ".##"]);
        let index = index(tile_cell(&map, 1, 1));
        assert_eq!(index - GRASS_EDGE_START, BLOB_INDEX[W as usize] as usize);
    }

    #[test]
    fn water_always_uses_water_variants() {
        let map = map_from(&["~~~", "~~~", "~~~"]);
        let cell = tile_cell(&map, 1, 1);
        assert_eq!(cell.row(), 0);
        assert!((WATER_START..WATER_START + WATER_VARIANTS).contains(&cell.col()));
    }

    fn sprite(col: usize, row: usize) -> GroundSprite {
        GroundSprite::new(SpriteCell::new(col, row), true)
    }

    #[test]
    fn frame_zero_keeps_the_original_cell() {
        assert_eq!(animated_cell(sprite(WATER_START, 0), 0, 0), SpriteCell::new(WATER_START, 0));
        assert_eq!(animated_cell(sprite(3, 2), 0, 0), SpriteCell::new(3, 2));
    }

    #[test]
    fn static_cells_never_animate() {
        for (col, row) in [(0, 0), (SAND_START, 0), (2, 4), (5, 6)] {
            for frame in 0..WATER_FRAMES {
                assert_eq!(animated_cell(sprite(col, row), frame, 3), SpriteCell::new(col, row));
            }
        }
    }

    #[test]
    fn animated_cells_stay_inside_the_sheet() {
        let sheet_rows = FRAMES_START_ROW + (WATER_FRAMES - 1) * FRAME_ROWS;
        for frame in 0..WATER_FRAMES {
            for phase in 0..WATER_FRAMES {
                for col in 0..SHEET_COLUMNS {
                    for row in 0..4 {
                        let cell = animated_cell(sprite(col, row), frame, phase);
                        assert!(cell.row() < sheet_rows);
                    }
                }
            }
        }
    }

    #[test]
    fn phase_offsets_open_water_but_not_shores() {
        let water = sprite(WATER_START, 0);
        let shore = sprite(3, 2);
        assert_ne!(animated_cell(water, 0, 1), animated_cell(water, 0, 0));
        assert_eq!(animated_cell(shore, 1, 0), animated_cell(shore, 1, 2));
    }

    #[test]
    fn tiles_flagged_static_ignore_the_frame() {
        let shore = GroundSprite::new(SpriteCell::new(3, 2), false);
        assert_eq!(animated_cell(shore, 2, 1), SpriteCell::new(3, 2));
    }

    #[test]
    fn sea_shores_animate_and_river_shores_do_not() {
        let map = map_from(&["~..rr", "~..rr", "~..rr"]);
        assert!(is_animated(&map, 1, 1));
        assert!(!is_animated(&map, 2, 1));
        assert!(is_animated(&map, 0, 1));
        assert!(is_animated(&map, 3, 1));
    }

    #[test]
    fn sand_touching_sea_and_river_animates() {
        let map = map_from(&["~.r"]);
        assert!(is_animated(&map, 1, 0));
    }

    #[test]
    fn diagonal_only_neighbor_picks_corner_piece() {
        let map = map_from(&["~..", "...", "..."]);
        let index = index(tile_cell(&map, 1, 1));
        assert_eq!(index - SHORE_START, BLOB_INDEX[NW as usize] as usize);
    }

    #[test]
    fn roads_connect_to_neighbors_and_default_to_vertical() {
        let mut map = map_from(&["###"]);
        map.build_road(1, 1);
        assert_eq!(road_cell(&map, 1, 1), Some(SpriteCell::new((N | S) as usize, 0)));
        map.build_road(2, 1);
        map.build_road(1, 2);
        assert_eq!(road_cell(&map, 1, 1), Some(SpriteCell::new((E | S) as usize, 0)));
        assert_eq!(road_cell(&map, 2, 1), Some(SpriteCell::new(W as usize, 0)));
        assert_eq!(road_cell(&map, 0, 0), None);
    }

    #[test]
    fn straight_streets_next_to_junctions_get_crosswalks() {
        let mut map = map_from(&["#######", "#######", "#######"]);
        for (x, y) in [(1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (3, 0), (3, 2)] {
            map.build_road(x, y);
        }
        assert_eq!(road_cell(&map, 1, 1), Some(SpriteCell::new(E as usize, 0)));
        assert_eq!(road_cell(&map, 2, 1), Some(SpriteCell::new(4, CROSSWALK_ROW)));
        assert_eq!(road_cell(&map, 4, 1), Some(SpriteCell::new(3, CROSSWALK_ROW)));
        assert_eq!(road_cell(&map, 3, 0), Some(SpriteCell::new(DEAD_END_CROSSWALKS + 1, CROSSWALK_ROW)));
        assert_eq!(road_cell(&map, 5, 1), Some(SpriteCell::new(W as usize, 0)));
        map.build_road(2, 2);
        assert_eq!(road_cell(&map, 2, 1), Some(SpriteCell::new((W | E | S) as usize, 0)));
    }

    #[test]
    fn roads_cannot_be_built_on_water_rocks_or_roads_and_replace_trees() {
        let mut map = map_from(&["~##"]);
        map.features[idx(1, 0)] = Some(Feature::Rock);
        map.features[idx(2, 0)] = Some(Feature::Tree);
        assert!(!map.can_build_road(0, 0));
        assert!(!map.can_build_road(1, 0));
        assert!(!map.can_build_road(-1, 0));
        assert!(map.build_road(2, 0));
        assert_eq!(map.feature(2, 0), None);
        assert!(!map.can_build_road(2, 0));
    }

    #[test]
    fn bridges_start_from_a_road_and_only_extend_straight() {
        let mut map = map_from(&["#~~~", "~~~~", "~~~~"]);
        assert!(!map.can_build_road(1, 0));
        map.build_road(0, 0);
        assert!(!map.can_build_road(0, 2));
        assert!(map.build_road(1, 0));
        assert_eq!(map.road(1, 0), Some(Road::Bridge(Axis::Horizontal)));
        assert!(!map.can_build_road(1, 1));
        assert!(map.can_build_road(2, 0));
        assert_eq!(road_cell(&map, 0, 0), Some(SpriteCell::new(E as usize, 0)));
        assert_eq!(road_cell(&map, 1, 0), Some(SpriteCell::new(1, BRIDGE_DECK_ROW)));
        assert_eq!(bridge_truss_cell(&map, 1, 0), Some(SpriteCell::new(1, BRIDGE_TRUSS_ROW)));
    }

    #[test]
    fn streets_do_not_link_to_the_side_of_a_bridge() {
        let mut map = map_from(&["#~#", "~~#"]);
        map.build_road(0, 0);
        map.build_road(1, 0);
        map.build_road(2, 1);
        map.build_road(2, 0);
        assert_eq!(road_cell(&map, 2, 0), Some(SpriteCell::new((W | S) as usize, 0)));
    }
}
