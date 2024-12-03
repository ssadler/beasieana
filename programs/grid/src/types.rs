
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq)]
#[repr(C, align(2))] // Ensure proper alignment for CellPos
pub struct CellPos {
    pub x: u16,
    pub y: u16,
    pub r: u16
}

impl CellPos {
    pub fn area(&self) -> u64 {
        (314159265 * (self.r as u64).pow(2)) / 100000000
    }
    pub fn overlaps(&self, o: &CellPos) -> bool {
        let dx = self.x as i32 - o.x as i32;
        let dy = self.y as i32 - o.y as i32;
        let distance_squared = dx * dx + dy * dy;
        let radius_sum = self.r as i32 + o.r as i32;
        distance_squared <= radius_sum * radius_sum
    }

    pub fn check_bounded(&self) {
        // Check lower bounds
        if self.x < self.r || self.y < self.r {
            panic!("OOB (lower)");
        }
        // Check upper bounds
        if u16::MAX-self.r < self.x || u16::MAX-self.r < self.y {
            panic!("OOB (upper)");
        }
    }

    // https://stackoverflow.com/a/402010
    pub fn overlaps_pad(&self, x: u16, y: u16, w: u16, h: u16) -> bool {

        // since we're working with integer division, we'll multiply the sizes of everything by 2

        let r32 = self.r as i32 * 2;
        let w32 = w as i32;
        let h32 = h as i32;

        // p[xy] is center point of pad
        let px = (x as i32)*2 + w32;
        let py = (y as i32)*2 + h32;

        //// d[xy] is distance from center points
        let dx = ((self.x as i32)*2 - px).abs();
        let dy = ((self.y as i32)*2 - py).abs();

        if dx >= w32 + r32 || dy >= h32 + r32 {
            false
        } else if dx <= w32 || dy <= h32 {
            true
        } else {
            (dx - w32).pow(2) + (dy - h32).pow(2) <= r32.pow(2)
        }
    }

    pub fn pads(&self, g: u16) -> Vec<(u16, u16)> {
        let gg = 2 << g;
        let xmin = (self.x - self.r) >> g;
        let xmax = (self.x + self.r) >> g;
        let ymin = (self.y - self.r) >> g;
        let ymax = (self.y + self.r) >> g;

        let mut out = Vec::new();

        for xx in xmin..(xmax+1) {
            for yy in ymin..(ymax+1) {
                if self.overlaps_pad(xx*gg, yy*gg, gg, gg) {
                    out.push((xx, yy));
                }
            }
        }

        out
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlaps_pad() {

        assert!(CellPos { x: 2, y: 2, r: 1 }.overlaps_pad(0, 0, 2, 3));
        assert!(!CellPos { x: 3, y: 3, r: 1 }.overlaps_pad(0, 0, 2, 3));
        assert!(!CellPos { x: 3, y: 3, r: 1 }.overlaps_pad(0, 0, 3, 2));

        assert!(!CellPos { x: 10, y: 10, r: 2 }.overlaps_pad(0, 0, 12, 8));
        assert!(CellPos { x: 10, y: 10, r: 2 }.overlaps_pad(0, 0, 12, 9));
    }
}


//  // 300-450
//  function cellOverlapsPad(Cell memory c, uint56 p) internal pure returns (bool) {
//
//    uint pg = 2**uint(Pad.g(p));
//
//    unchecked {
//      uint px = Pad.x(p);
//      assembly { px := add(mul(px, pg), div(pg, 2)) }
//      uint py = Pad.y(p);
//      assembly { py := add(mul(py, pg), div(pg, 2)) }
//
//      uint dx = c.x - px;
//      assembly { if slt(dx, 0) { dx := mul(dx, sub(0, 1)) } }
//      uint dy = c.y - py;
//      assembly { if slt(dy, 0) { dy := mul(dy, sub(0, 1)) } }
//      
//      if (dx > pg/2 + c.r) { return false; }
//      if (dy > pg/2 + c.r) { return false; }
//
//      if (dx <= pg/2) { return true; }
//      if (dy <= pg/2) { return true; }
//
//      uint cornerSq = (dx - pg/2)**2 + (dy - pg/2)**2;
//
//      return cornerSq <= (uint(c.r)**2);
//    }
//  }
