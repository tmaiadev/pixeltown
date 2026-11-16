#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuildingKind {
    CoalPlant,
}

impl BuildingKind {
    // Width and height in tiles.
    pub const fn footprint(self) -> (usize, usize) {
        match self {
            BuildingKind::CoalPlant => (5, 5),
        }
    }

    // Top-left tile of the footprint when the building is centered on `center`.
    pub fn origin(self, center: (i32, i32)) -> (i32, i32) {
        let (w, h) = self.footprint();
        (center.0 - (w / 2) as i32, center.1 - (h / 2) as i32)
    }

    // Every tile of the footprint, row by row.
    pub fn tiles(self, origin: (i32, i32)) -> impl Iterator<Item = (i32, i32)> {
        let (w, h) = self.footprint();
        (0..h as i32).flat_map(move |dy| (0..w as i32).map(move |dx| (origin.0 + dx, origin.1 + dy)))
    }
}

// A building placed on the map, identified by the top-left tile of its footprint.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Building {
    pub kind: BuildingKind,
    pub x: u16,
    pub y: u16,
}

impl Building {
    pub fn tiles(self) -> impl Iterator<Item = (i32, i32)> {
        self.kind.tiles((self.x as i32, self.y as i32))
    }
}

// What a tile knows about the building on it: which one, and where the tile sits in its footprint.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BuildingTile {
    pub kind: BuildingKind,
    pub dx: u8,
    pub dy: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coal_plant_covers_five_by_five_tiles() {
        let tiles: Vec<_> = BuildingKind::CoalPlant.tiles((10, 20)).collect();
        assert_eq!(tiles.len(), 25);
        assert_eq!(tiles[0], (10, 20));
        assert_eq!(tiles[24], (14, 24));
    }

    #[test]
    fn centering_puts_the_middle_tile_under_the_cursor() {
        assert_eq!(BuildingKind::CoalPlant.origin((10, 10)), (8, 8));
    }
}
