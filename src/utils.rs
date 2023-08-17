use std::collections::VecDeque;

pub fn adjust_padding(buf: &mut Vec<u8>, align: usize) {
    if align == 1 {
        return; // no padding is required for 1-alignment
    }
    let len = buf.len();
    let required_padding = (align - (len % align)) % align;
    for _ in 0..required_padding {
        buf.push(0);
    }
}

pub fn align_start(buf: &mut VecDeque<u8>, align: usize) {
    let len = buf.len();
    let padding = (align - (len % align)) % align;
    for _ in 0..padding {
        if buf[0] != 0 {
            panic!("padding must be 0, which was not");
        }
        buf.pop_front();
    }
}

pub fn consume_null(buf: &mut VecDeque<u8>) {
    if buf.len() == 0 {
        return;
    }
    while buf.len() > 0 && buf[0] == 0 {
        buf.pop_front();
    }
}

pub fn align_counter(ctr: &mut usize, align: usize) {
    if *ctr % align != 0 {
        // adjust counter for 4 align
        *ctr += (align - (*ctr % align)) % align;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! vecdeque {
        ($v:tt) => {
            VecDeque::from($v)
        };
    }

    #[test]
    fn test_adjust_padding() {
        let mut buf = vec![];

        // empty buffer is all aligned
        adjust_padding(&mut buf, 1);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf, vec![]);
        adjust_padding(&mut buf, 3);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf, vec![]);
        adjust_padding(&mut buf, 8);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf, vec![]);

        let mut buf = vec![1, 2, 3, 4];

        // align 1 should never change buffer, as everything is 1 byte aligned
        adjust_padding(&mut buf, 1);
        assert_eq!(buf.len(), 4);
        assert_eq!(buf, vec![1, 2, 3, 4]);

        // 4 aligned buffer should not have any changes
        adjust_padding(&mut buf, 4);
        assert_eq!(buf.len(), 4);
        assert_eq!(buf, vec![1, 2, 3, 4]);

        adjust_padding(&mut buf, 3);
        assert_eq!(buf.len(), 6);
        assert_eq!(buf, vec![1, 2, 3, 4, 0, 0]);

        let mut buf = vec![1, 2, 3, 4];
        adjust_padding(&mut buf, 8);
        assert_eq!(buf.len(), 8);
        assert_eq!(buf, vec![1, 2, 3, 4, 0, 0, 0, 0]);

        let mut buf = vec![1, 2, 3];
        adjust_padding(&mut buf, 4);
        assert_eq!(buf.len(), 4);
        assert_eq!(buf, vec![1, 2, 3, 0]);
    }

    #[test]
    fn test_align_start() {
        let mut buf = vecdeque!([0, 1, 2]);
        align_start(&mut buf, 2);
        assert_eq!(buf.len(), 2);
        assert_eq!(buf, vecdeque!([1, 2]));

        let mut buf = vecdeque!([0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2]);
        align_start(&mut buf, 8);
        assert_eq!(buf.len(), 6);
        assert_eq!(buf, vecdeque!([0, 0, 0, 0, 1, 2]));

        let mut buf = vecdeque!([0, 1, 2]);
        align_start(&mut buf, 1);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf, vecdeque!([0, 1, 2]));
    }

    #[test]
    fn test_inverse() {
        let mut buf = vec![1, 2];
        adjust_padding(&mut buf, 4);
        let mut inverse = VecDeque::from(buf);
        inverse.pop_front();
        inverse.pop_front();
        inverse.push_back(3);
        inverse.push_back(4);
    }

    #[test]
    #[should_panic]
    fn test_align_start_panic() {
        let mut buf = vecdeque!([1, 1, 2]);
        align_start(&mut buf, 2);
        assert!(false);
    }
}
