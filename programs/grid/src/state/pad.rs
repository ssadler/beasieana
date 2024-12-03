
use std::cell::*;

use crate::types::CellPos;

macro_rules! write_bytes {
    ($data:expr, $start:expr, $value:expr) => {
        $data[$start..$start + std::mem::size_of_val(&$value)].copy_from_slice(&$value.to_le_bytes());
    };
}


struct PadStorage<'a>(&'a mut [u8]);

static HEADER_SIZE: usize = 4;
static ITEM_SIZE: usize = 12;


impl<'a> PadStorage<'a> {
    #[inline]
    fn off(i: usize) -> usize {
        4 + i * 12
    }
    pub fn len(&self) -> usize {
        return self.0[0] as usize;
    }
    pub fn iter_cell_id(&self) -> impl Iterator<Item = &u32> + '_ {
        (0..self.len()).map(|i| unsafe { &*(self.0.as_ptr().add(Self::off(i)) as *const u32) })
    }
    pub fn iter_cell_pos(&self) -> impl Iterator<Item = &CellPos> + '_ {
        (0..self.len()).map(|i| unsafe { &*(self.0.as_ptr().add(Self::off(i)+4) as *const CellPos) })
    }

    pub fn get(&self, idx: usize) -> (u32, &CellPos) {
        assert!(idx < self.len(), "get: idx oob");
        unsafe {
            (
                *(self.0.as_ptr().add(Self::off(idx)) as *const u32),
                &*(self.0.as_ptr().add(Self::off(idx)+4) as *const CellPos),
            )
        }
    }

    pub fn append(&mut self, cell_id: u32, pos: &CellPos) {
        let len = self.len();
        let off = len * ITEM_SIZE + HEADER_SIZE;
        // write cell_id
        write_bytes!(self.0, off, cell_id);
        // write CellPos
        unsafe {
            let ptr = self.0.as_mut_ptr().add(off+4) as *mut CellPos;
            *ptr = *pos; // Write the CellPos directly into the buffer
        }
        // update length
        self.0[0] += 1;
    }
    pub fn remove(&mut self, idx: usize) {
        let len = self.len();
        assert!(idx < len, "remove: idx oob");

        let r = Self::off(len-1)..Self::off(len);

        // if not last, copy
        if idx < len + 1 {
            self.0.copy_within(r.clone(), Self::off(idx));
        }

        // scrub last
        self.0[r].fill(0);

        // update length
        self.0[0] -= 1;
    }
}


pub fn add_cell_to_pad(mut data: RefMut<&mut [u8]>, cell_id: u32, pos: &CellPos) {
    // 2 bytes for len
    // 4 byte cell id + 6 byte cell pos = 10 bytes per item

    let mut storage = PadStorage(&mut data);

    for other_pos in storage.iter_cell_pos() {
        if pos.overlaps(other_pos) {
            panic!("overlaps");
        }
    }

    storage.append(cell_id, pos);
}




pub fn remove_cell_from_pad(mut data: RefMut<&mut [u8]>, cell_id: u32) {

    let mut storage = PadStorage(&mut data);
    let mut midx = None;

    for (i, other_id) in storage.iter_cell_id().enumerate() {
        if &cell_id == other_id {
            midx = Some(i);
            break;
        }
    }

    if let Some(idx) = midx {
        storage.remove(idx);
    } else {
        panic!("cell not found in pad");
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "overlaps")]
    fn test_add_cell_to_pad_overlap() {
        let mut buffer = vec![0; 32]; // create buffer for 3 cells, 10 bytes per cell plus 2 for len
        let mut_ref = RefCell::new(&mut buffer[..]);
        let cell_pos = CellPos { x: 10, y: 10, r: 5 };
        add_cell_to_pad(mut_ref.borrow_mut(), 1, &cell_pos); // Add the first cell
        add_cell_to_pad(mut_ref.borrow_mut(), 1, &cell_pos); // Attempt again
    }

    #[test]
    fn test_add_cell_to_pad_no_overlap() {
        let mut buffer = vec![0; 240]; // Create a buffer to hold 3 cells (80 bytes per cell)
        let mut_ref = RefCell::new(&mut buffer[..]);
        let p0 = CellPos { x: 10, y: 10, r: 5 };
        add_cell_to_pad(mut_ref.borrow_mut(), 1, &p0); // Add the first cell
        let p1 = CellPos { x: 20, y: 20, r: 5 };
        add_cell_to_pad(mut_ref.borrow_mut(), 2, &p1); // Attempt again

        let mut storage = PadStorage(&mut buffer);

        // test first
        let (cid, p) = storage.get(0);
        assert!(cid == 1);
        assert!(p == &p0);

        // test second
        let (cid, p) = storage.get(1);
        assert!(cid == 2);
        assert!(p == &p1);

        // delete second
        storage.remove(1);
        let (cid, p) = storage.get(0);
        assert!(cid == 1);
        assert!(p == &p0);
        assert!(storage.len() == 1);

        // delete first
        storage.remove(0);
        assert!(storage.len() == 0);
    }
}
