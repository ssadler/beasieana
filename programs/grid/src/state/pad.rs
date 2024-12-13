use std::{cell::*, ops::{Index, IndexMut}};

use crate::{types::CellPos, CellPositionedId};


type PadData<'a> = std::rc::Rc<RefCell<&'a mut [u8]>>; 

pub struct PadStorage<'a> {
    pub data: PadData<'a>,
    ptr: *mut u8
}

static HEADER_SIZE: usize = 4;
static ITEM_SIZE: usize = 12;


impl<'a> PadStorage<'a> {
    pub fn new(data: PadData<'a>) -> PadStorage<'a> {
        let ptr = unsafe { (*data.as_ptr()).as_mut_ptr() };
        PadStorage { data, ptr }
    }
    #[inline]
    fn off(i: usize) -> usize {
        HEADER_SIZE + i * ITEM_SIZE
    }
    #[inline]
    fn _get(&self, idx: usize) -> &'a CellPositionedId {
        unsafe { &*(self.ptr.add(Self::off(idx)) as *const CellPositionedId) }
    }
    #[inline]
    pub fn len(&self) -> usize {
        return unsafe { *self.ptr } as usize;
    }
    pub fn slice(&self) -> &'a [CellPositionedId] {
        unsafe {
            &*std::ptr::slice_from_raw_parts(self.ptr.add(4) as *const CellPositionedId, self.len())
        }
    }
    pub fn slice_mut(&mut self) -> &'a mut [CellPositionedId] {
        unsafe {
            &mut *std::ptr::slice_from_raw_parts_mut(self.ptr.add(4) as *mut CellPositionedId , self.len())
        }
    }
    pub fn get(&self, idx: usize) -> &'a CellPositionedId {
        assert!(idx < self.len(), "get: idx oob");
        self._get(idx)
    }

    pub fn append(&mut self, cell_id: u32, pos: &CellPos) {
        let off = Self::off(self.len());
        unsafe { *(self.ptr.add(off) as *mut (u32, CellPos)) = (cell_id, *pos); };
        unsafe { *self.ptr += 1; }
    }

    pub fn remove(&mut self, idx: usize) {
        unsafe {
            let len = *self.ptr as usize;
            assert!(idx < len, "remove: idx oob");
            *self.ptr -= 1;

            let p_last = self.ptr.add(Self::off(len-1));

            // if not last, copy
            if idx + 1 < len {
                self[idx] = self[len-1];
            }

            // scrub last
            p_last.write_bytes(0, ITEM_SIZE);
        }
    }

    pub fn remove_cell(&mut self, cell_id: u32) {
        let midx = self.slice().iter().position(|p| p.cell_id == cell_id);
        self.remove(midx.expect("remove_cell: not found"));
    }
    pub fn update_cell(&mut self, other: &CellPositionedId) {
        for cell in self.slice_mut().iter_mut() {
            if cell.cell_id == other.cell_id {
                cell.pos = other.pos;
                return;
            }
        }
        panic!("update_cell: not found");
    }
}

impl<'a> Index<usize> for PadStorage<'a> {
    type Output = CellPositionedId;
    fn index(&self, idx: usize) -> &Self::Output {
        unsafe { &*(self.ptr.add(Self::off(idx)) as *const CellPositionedId) }
    }
}
impl<'a> IndexMut<usize> for PadStorage<'a> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        unsafe { &mut *(self.ptr.add(Self::off(idx)) as *mut CellPositionedId) }
    }
}


#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;


    //#[test]
    //#[should_panic(expected = "overlaps")]
    //fn test_add_cell_to_pad_overlap() {
    //    let mut buffer = vec![0; 32]; // create buffer for 3 cells, 10 bytes per cell plus 2 for len
    //    let mut_ref = RefCell::new(&mut buffer[..]);
    //    let cell_pos = CellPos { x: 10, y: 10, r: 5 };
    //    add_cell_to_pad(mut_ref.borrow_mut(), 1, &cell_pos); // Add the first cell
    //    add_cell_to_pad(mut_ref.borrow_mut(), 1, &cell_pos); // Attempt again
    //}

    #[test]
    fn test_add_cell_to_pad_no_overlap() {
        let mut buffer = vec![0; 240]; // Create a buffer to hold 3 cells (80 bytes per cell)
        let mut storage = PadStorage::new(Rc::new(RefCell::new(&mut buffer)));
        let p0 = CellPos { x: 10, y: 10, r: 5 };
        storage.append(1, &p0);
        let p1 = CellPos { x: 20, y: 20, r: 5 };
        storage.append(2, &p1); // Attempt again


        // test first
        let (cid, p) = storage.get(0).into();
        assert!(cid == 1);
        assert!(p == p0);

        // test second
        let (cid, p) = storage.get(1).into();
        assert!(cid == 2);
        assert!(p == p1);

        // delete second
        storage.remove(1);
        let (cid, p) = storage.get(0).into();
        assert!(cid == 1);
        assert!(p == p0);
        assert!(storage.len() == 1);

        // delete first
        storage.remove(0);
        assert!(storage.len() == 0);
    }
}
