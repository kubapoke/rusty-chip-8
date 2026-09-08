fn get_nibbles(num: &u16, first: usize, last: usize) -> u16 {
    let mut mask: u16 = 0;
    for i in first..=last {
        let shift = i as u16;
        mask |= 0xf000 >> (4 * shift);
    }
    *num & mask
}

fn get_nibble(num: &u16, index: usize) -> u16 {
    get_nibbles(num, index, index)
}

fn get_nibble_reduced(num: &u16, index: usize) -> u16 {
    get_nibble(num, index) >> (4 * (3 - index))
}

pub fn get_code(command: &u16) -> u16 {
    get_nibble_reduced(command, 0)
}

pub fn get_x(command: &u16) -> usize {
    get_nibble_reduced(command, 1) as usize
}

pub fn get_y(command: &u16) -> usize {
    get_nibble_reduced(command, 2) as usize
}

pub fn get_n(command: &u16) -> u8 {
    get_nibble(command, 3) as u8
}

pub fn get_nn(command: &u16) -> u8 {
    get_nibbles(command, 2, 3) as u8
}

pub fn get_nnn(command: &u16) -> u16 {
    get_nibbles(command, 1, 3)
}

pub fn extract_values(command: &u16) -> (u16, usize, usize, u8, u8, u16) {
    (
        get_code(command),
        get_x(command),
        get_y(command),
        get_n(command),
        get_nn(command),
        get_nnn(command),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_nibbles() {
        assert_eq!(get_nibbles(&0x0000, 0, 3), 0x0000);
        assert_eq!(get_nibbles(&0xffff, 0, 3), 0xffff);
        assert_eq!(get_nibbles(&0xffff, 1, 2), 0x0ff0);
        assert_eq!(get_nibbles(&0xffff, 0, 0), 0xf000);
    }

    #[test]
    fn test_get_nibble() {
        assert_eq!(get_nibble(&0x0000, 0), 0x0000);
        assert_eq!(get_nibble(&0xffff, 0), 0xf000);
        assert_eq!(get_nibble(&0xffff, 1), 0x0f00);
        assert_eq!(get_nibble(&0xffff, 2), 0x00f0);
        assert_eq!(get_nibble(&0xffff, 3), 0x000f);
    }

    #[test]
    fn test_get_nibble_reduced() {
        assert_eq!(get_nibble_reduced(&0x0000, 0), 0x0000);
        assert_eq!(get_nibble_reduced(&0x1234, 0), 0x0001);
        assert_eq!(get_nibble_reduced(&0x1234, 1), 0x0002);
        assert_eq!(get_nibble_reduced(&0x1234, 2), 0x0003);
        assert_eq!(get_nibble_reduced(&0x1234, 3), 0x0004);
    }
}
