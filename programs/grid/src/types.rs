

pub struct CellPos {
    pub x: u16,
    pub y: u16,
    pub r: u16
}

impl CellPos {
    pub fn overlaps(&self, x: u16, y: u16, r: u16) -> bool {
        let dx = self.x as i32 - x as i32;
        let dy = self.y as i32 - y as i32;
        let distance_squared = dx * dx + dy * dy;
        let radius_sum = self.r as i32 + r as i32;
        distance_squared <= radius_sum * radius_sum
    }
}
