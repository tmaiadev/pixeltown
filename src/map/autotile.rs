use super::{Map, Terrain, MAP_SIZE};

pub const SHEET_COLUMNS: usize = 16;
const GRASS_VARIANTS: usize = 4;
const SAND_VARIANTS: usize = 3;
const WATER_VARIANTS: usize = 3;
const SAND_START: usize = GRASS_VARIANTS;
const WATER_START: usize = SAND_START + SAND_VARIANTS;
const SHORE_START: usize = SHEET_COLUMNS;
const GRASS_EDGE_START: usize = 4 * SHEET_COLUMNS;

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

fn neighbor_mask(map: &Map, x: usize, y: usize, other: Terrain) -> u8 {
    let (x, y) = (x as i32, y as i32);
    let mut mask = 0;
    for (dx, dy, bit) in CARDINALS.into_iter().chain(DIAGONALS) {
        if map.get_or(x + dx, y + dy, map.get(x as usize, y as usize)) == other {
            mask |= bit;
        }
    }
    normalize(mask)
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

pub fn tile_cells(map: &Map) -> Vec<(u8, u8)> {
    let mut cells = Vec::with_capacity(MAP_SIZE * MAP_SIZE);
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let (col, row) = tile_cell(map, x, y);
            cells.push((col as u8, row as u8));
        }
    }
    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_from(rows: &[&str]) -> Map {
        let mut tiles = vec![Terrain::Grass; MAP_SIZE * MAP_SIZE];
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                tiles[y * MAP_SIZE + x] = match c {
                    '~' => Terrain::Water,
                    '.' => Terrain::Sand,
                    _ => Terrain::Grass,
                };
            }
        }
        Map { tiles }
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
    fn blob_indices_fit_in_their_sheet_sections() {
        assert!(SHORE_START + BLOB_COUNT <= GRASS_EDGE_START);
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

    #[test]
    fn diagonal_only_neighbor_picks_corner_piece() {
        let map = map_from(&["~..", "...", "..."]);
        let (col, row) = tile_cell(&map, 1, 1);
        let index = row * SHEET_COLUMNS + col;
        assert_eq!(index - SHORE_START, BLOB_INDEX[NW as usize] as usize);
    }
}
