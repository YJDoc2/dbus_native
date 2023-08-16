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