use super::{Feature, Map, Terrain, MAP_SIZE};

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
    let own = map.get(x, y);
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

fn cell(index: usize) -> (usize, usize) {
    (index % SHEET_COLUMNS, index / SHEET_COLUMNS)
}

pub fn tile_cell(map: &Map, x: usize, y: usize) -> (usize, usize) {
    match map.get(x, y) {
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

pub fn feature_cell(map: &Map, x: usize, y: usize) -> Option<(usize, usize)> {
    let index = match map.feature(x, y)? {
        Feature::Tree => TREE_START + variant(x, y, FEATURE_VARIANTS),
        Feature::Rock => ROCK_START + variant(x, y, FEATURE_VARIANTS),
        Feature::Forest => {
            // The canopy texture spans FOREST_PERIOD tiles, so pick the slice by position.
            let blob = BLOB_INDEX[forest_edge_mask(map, x, y) as usize] as usize;
            let position = (y % FOREST_PERIOD) * FOREST_PERIOD + x % FOREST_PERIOD;
            FOREST_START + blob * FOREST_PERIOD * FOREST_PERIOD + position
        }
    };
    Some(cell(index))
}

pub fn feature_cells(map: &Map) -> Vec<Option<(u8, u8)>> {
    let mut cells = Vec::with_capacity(MAP_SIZE * MAP_SIZE);
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            cells.push(feature_cell(map, x, y).map(|(col, row)| (col as u8, row as u8)));
        }
    }
    cells
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tile {
    cell: (u8, u8),
    animated: bool,
}

impl Tile {
    fn new((col, row): (usize, usize), animated: bool) -> Self {
        Self {
            cell: (col as u8, row as u8),
            animated,
        }
    }

    pub fn is_animated(&self) -> bool {
        self.animated
    }
}

fn is_animated(map: &Map, x: usize, y: usize) -> bool {
    match map.get(x, y) {
        Terrain::Water => true,
        Terrain::Sand => {
            let (x, y) = (x as i32, y as i32);
            CARDINALS.into_iter().chain(DIAGONALS).any(|(dx, dy, _)| {
                let (nx, ny) = (x + dx, y + dy);
                Map::in_bounds(nx, ny)
                    && map.get(nx as usize, ny as usize) == Terrain::Water
                    && !map.is_river(nx as usize, ny as usize)
            })
        }
        Terrain::Grass => false,
    }
}

// `phase` desyncs open water between tiles; shores always follow the global frame.
pub fn animated_cell(tile: Tile, frame: usize, phase: usize) -> (usize, usize) {
    let (col, row) = (tile.cell.0 as usize, tile.cell.1 as usize);
    let open_water = row == 0 && (WATER_START..WATER_START + WATER_VARIANTS).contains(&col);
    let shore = (1..FRAME_ROWS).contains(&row);
    let frame = match (tile.animated, open_water, shore) {
        (true, true, _) => (frame + phase) % WATER_FRAMES,
        (true, _, true) => frame % WATER_FRAMES,
        _ => 0,
    };
    let row = match frame {
        0 => row,
        f => FRAMES_START_ROW + (f - 1) * FRAME_ROWS + row,
    };
    (col, row)
}

pub fn tile_cells(map: &Map) -> Vec<Tile> {
    let mut tiles = Vec::with_capacity(MAP_SIZE * MAP_SIZE);
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            tiles.push(Tile::new(tile_cell(map, x, y), is_animated(map, x, y)));
        }
    }
    tiles
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::idx;

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
            tiles,
            river,
            features,
        }
    }

    #[test]
    fn features_pick_cells_from_their_own_section() {
        let mut map = map_from(&["###"]);
        assert_eq!(feature_cell(&map, 0, 0), None);
        for (feature, start) in [(Feature::Tree, TREE_START), (Feature::Rock, ROCK_START)] {
            for x in 0..3 {
                map.features[idx(x, 0)] = Some(feature);
                let (col, row) = feature_cell(&map, x, 0).unwrap();
                assert_eq!(row, 0);
                assert!((start..start + FEATURE_VARIANTS).contains(&col));
            }
        }
    }

    fn forest_index(map: &Map, x: usize, y: usize) -> usize {
        let (col, row) = feature_cell(map, x, y).unwrap();
        row * SHEET_COLUMNS + col - FOREST_START
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
        let (col, row) = tile_cell(&flat, 1, 1);
        assert_eq!(row, 0);
        assert!((SAND_START..WATER_START).contains(&col));
    }

    #[test]
    fn sand_next_to_water_uses_shore_section() {
        let map = map_from(&["~~~", "~..", "~.."]);
        let (col, row) = tile_cell(&map, 1, 1);
        let index = row * SHEET_COLUMNS + col;
        assert!((SHORE_START..GRASS_EDGE_START).contains(&index));
        assert_eq!(index - SHORE_START, BLOB_INDEX[(N | W) as usize] as usize);
    }

    #[test]
    fn grass_next_to_sand_uses_grass_edge_section() {
        let map = map_from(&[".##", ".##", ".##"]);
        let (col, row) = tile_cell(&map, 1, 1);
        let index = row * SHEET_COLUMNS + col;
        assert_eq!(index - GRASS_EDGE_START, BLOB_INDEX[W as usize] as usize);
    }

    #[test]
    fn water_always_uses_water_variants() {
        let map = map_from(&["~~~", "~~~", "~~~"]);
        let (col, row) = tile_cell(&map, 1, 1);
        assert_eq!(row, 0);
        assert!((WATER_START..WATER_START + WATER_VARIANTS).contains(&col));
    }

    fn tile(cell: (usize, usize)) -> Tile {
        Tile::new(cell, true)
    }

    #[test]
    fn frame_zero_keeps_the_original_cell() {
        let water = (WATER_START, 0);
        let shore = (3, 2);
        assert_eq!(animated_cell(tile(water), 0, 0), water);
        assert_eq!(animated_cell(tile(shore), 0, 0), shore);
    }

    #[test]
    fn static_cells_never_animate() {
        for cell in [(0, 0), (SAND_START, 0), (2, 4), (5, 6)] {
            for frame in 0..WATER_FRAMES {
                assert_eq!(animated_cell(tile(cell), frame, 3), cell);
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
                        let (_, r) = animated_cell(tile((col, row)), frame, phase);
                        assert!(r < sheet_rows);
                    }
                }
            }
        }
    }

    #[test]
    fn phase_offsets_open_water_but_not_shores() {
        let water = tile((WATER_START, 0));
        let shore = tile((3, 2));
        assert_ne!(animated_cell(water, 0, 1), animated_cell(water, 0, 0));
        assert_eq!(animated_cell(shore, 1, 0), animated_cell(shore, 1, 2));
    }

    #[test]
    fn tiles_flagged_static_ignore_the_frame() {
        let shore = Tile::new((3, 2), false);
        assert_eq!(animated_cell(shore, 2, 1), (3, 2));
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
        let (col, row) = tile_cell(&map, 1, 1);
        let index = row * SHEET_COLUMNS + col;
        assert_eq!(index - SHORE_START, BLOB_INDEX[NW as usize] as usize);
    }
}
