
use std::cell::*;
use crate::types::CellPos;


macro_rules! write_bytes {
    ($data:expr, $start:expr, $value:expr) => {
        $data[$start..$start + std::mem::size_of_val(&$value)].copy_from_slice(&$value.to_le_bytes());
    };
}
macro_rules! read_u16 {
    ($data:expr, $offset:expr) => {
        unsafe { *(($data).as_ptr().add($offset) as *const u16) }
    };
}
macro_rules! read_u32 {
    ($data:expr, $offset:expr) => {
        unsafe { *(($data).as_ptr().add($offset) as *const u32) }
    };
}

pub fn add_cell_to_pad(mut data: RefMut<&mut [u8]>, cell_id: u32, pos: &CellPos) {
    // cell id 32
    // cell pos 48
    let size = 80;

    let max = data.len() / size;

    // read length
    let len = read_u16!(data, 0) as usize;
    if len == max {
        panic!("no space left in pad");
    }

    for idx in 0..len {
        let i = idx * size;
        let x = read_u16!(data, i+4);
        let y = read_u16!(data, i+6);
        let r = read_u16!(data, i+8);

        if pos.overlaps(x, y, r) {
            panic!("overlaps");
        }
    }

    // write
    let off = len * size;
    write_bytes!(data, off, cell_id);
    write_bytes!(data, off+4, pos.x);
    write_bytes!(data, off+6, pos.y);
    write_bytes!(data, off+8, pos.r);

    // update len
    write_bytes!(data, 0, (len+1) as u16);
}

pub fn remove_cell_from_pad(mut data: RefMut<&mut [u8]>, cell_id: u32) {
    // cell id 32
    // cell pos 48
    let size = 80;

    let max = data.len() / size;

    // read length
    let len = read_u16!(data, 0) as usize;
    if len == max {
        panic!("no space left in pad");
    }

    for idx in 0..len {
        let id = read_u32!(data, idx * size);

        if id == cell_id {
            // TODO
            // update len
            write_bytes!(data, 0, (len-1) as u16);
            return;
        }
    }

    panic!("cell not found in pad");
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "overlaps")]
    fn test_add_cell_to_pad_overlap() {
        let mut buffer = vec![0; 240]; // Create a buffer to hold 3 cells (80 bytes per cell)
        let mut_ref = RefCell::new(&mut buffer[..]);
        let cell_pos = CellPos { x: 10, y: 10, r: 5 };
        add_cell_to_pad(mut_ref.borrow_mut(), 1, &cell_pos); // Add the first cell
        add_cell_to_pad(mut_ref.borrow_mut(), 1, &cell_pos); // Attempt again
    }

    #[test]
    fn test_add_cell_to_pad_no_overlap() {
        let mut buffer = vec![0; 240]; // Create a buffer to hold 3 cells (80 bytes per cell)
        let mut_ref = RefCell::new(&mut buffer[..]);
        let cell_pos = CellPos { x: 10, y: 10, r: 5 };
        add_cell_to_pad(mut_ref.borrow_mut(), 1, &cell_pos); // Add the first cell
        let cell_pos = CellPos { x: 20, y: 20, r: 5 };
        add_cell_to_pad(mut_ref.borrow_mut(), 1, &cell_pos); // Attempt again
    }
}
