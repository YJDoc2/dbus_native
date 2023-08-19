#[derive(Debug)]
pub enum DbusError {
    IncompleteImplementation(String),
    IncorrectMessage(String),
    ConnectionError(String),
}

pub type Result<T> = std::result::Result<T, DbusError>;

impl From<nix::Error> for DbusError {
    fn from(err: nix::Error) -> DbusError {
        DbusError::ConnectionError(err.to_string())
    }
}

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

pub fn align_counter(ctr: &mut usize, align: usize) {
    if *ctr % align != 0 {
        // adjust counter for 4 align
        *ctr += (align - (*ctr % align)) % align;
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
}
