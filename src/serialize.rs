use super::utils::adjust_padding;
pub trait DbusSerialize {
    fn get_signature() -> String
    where
        Self: Sized;
    fn serialize(&self, buf: &mut Vec<u8>);
}

pub struct Variant<T>(pub T);

pub struct Structure {
    key: String,
    val: Box<dyn DbusSerialize>,
}

impl DbusSerialize for () {
    fn get_signature() -> String {
        unreachable!("should never reach here");
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        unreachable!("should never reach here");
    }
}

impl<T1: DbusSerialize, T2: DbusSerialize> DbusSerialize for (T1, T2) {
    fn get_signature() -> String {
        format!("{}{}", T1::get_signature(), T2::get_signature())
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.0.serialize(buf);
        self.1.serialize(buf);
    }
}

impl DbusSerialize for String {
    fn get_signature() -> String {
        "s".to_string()
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        adjust_padding(buf, 4);
        let length = self.len() as u32;
        buf.extend_from_slice(&length.to_le_bytes());

        buf.extend_from_slice(self.as_bytes());
        buf.push(0); // needs to be null terminated
    }
}

impl DbusSerialize for bool {
    fn get_signature() -> String {
        "b".to_string()
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        adjust_padding(buf, 4);
        let val: u32 = match self {
            true => 1,
            false => 0,
        };
        buf.extend_from_slice(&val.to_le_bytes());
    }
}

impl DbusSerialize for u16 {
    fn get_signature() -> String {
        "q".to_string()
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        adjust_padding(buf, 2);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl DbusSerialize for u32 {
    fn get_signature() -> String {
        "u".to_string()
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        adjust_padding(buf, 4);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl DbusSerialize for u64 {
    fn get_signature() -> String {
        "t".to_string()
    }

    fn serialize(&self, buf: &mut Vec<u8>) {
        adjust_padding(buf, 8);
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl<T: DbusSerialize> DbusSerialize for Vec<T> {
    fn get_signature() -> String {
        let sub_type = T::get_signature();
        format!("a{}", sub_type)
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        adjust_padding(buf, 4);
        let len = self.len() as u32;
        buf.extend_from_slice(&len.to_le_bytes());
        for elem in self.iter() {
            elem.serialize(buf);
        }
    }
}

impl<T: DbusSerialize> DbusSerialize for Variant<T> {
    fn get_signature() -> String {
        "v".to_string()
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        // no alignment needed, as variant is 1-align
        let sub_type = T::get_signature();
        let signature_length = sub_type.len() as u8; // signature length must be < 256
        buf.push(signature_length);
        buf.extend_from_slice(sub_type.as_bytes());
        buf.push(0);
        self.0.serialize(buf);
    }
}
impl DbusSerialize for Structure {
    fn get_signature() -> String {
        "(sv)".to_string()
    }
    fn serialize(&self, buf: &mut Vec<u8>) {
        adjust_padding(buf, 8);
        self.key.serialize(buf);
        self.val.serialize(buf);
    }
}
