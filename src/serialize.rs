use super::utils::{adjust_padding, align_counter};

pub trait DbusSerialize {
    fn get_signature() -> String
    where
        Self: Sized;
    fn serialize(&self, buf: &mut Vec<u8>);
    fn deserialize(buf: &[u8], counter: &mut usize) -> Self
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct Variant<T>(pub T);

pub struct Structure {
    key: String,
    val: Box<dyn DbusSerialize>,
}

impl DbusSerialize for () {
    fn get_signature() -> String {
        String::new()
    }
    fn serialize(&self, _: &mut Vec<u8>) {}
    // for (), we have to ignore body , so we simply clear it out
    fn deserialize(buf: &[u8], counter: &mut usize) -> Self {
        *counter = buf.len();
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
    fn deserialize(buf: &[u8], counter: &mut usize) -> Self {
        let t1 = T1::deserialize(buf, counter);
        let t2 = T2::deserialize(buf, counter);
        (t1, t2)
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
    fn deserialize(buf: &[u8], counter: &mut usize) -> Self {
        align_counter(counter, 4);
        let length = u32::from_le_bytes(buf[*counter..*counter + 4].try_into().unwrap()) as usize;
        *counter += 4;
        let ret = String::from_utf8((&buf[*counter..*counter + length]).into()).unwrap();
        *counter += length + 1; // +1 accounting for null
        ret
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
    fn deserialize(buf: &[u8], counter: &mut usize) -> Self {
        align_counter(counter, 4);
        let ret = u32::from_le_bytes(buf[*counter..*counter + 4].try_into().unwrap());
        *counter += 4;
        ret != 0
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
    fn deserialize(buf: &[u8], counter: &mut usize) -> Self {
        align_counter(counter, 2);
        let ret = u16::from_le_bytes(buf[*counter..*counter + 2].try_into().unwrap());
        *counter += 2;
        ret
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
    fn deserialize(buf: &[u8], counter: &mut usize) -> Self {
        align_counter(counter, 4);
        let ret = u32::from_le_bytes(buf[*counter..*counter + 4].try_into().unwrap());
        *counter += 4;
        ret
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
    fn deserialize(buf: &[u8], counter: &mut usize) -> Self {
        align_counter(counter, 8);
        let ret = u64::from_le_bytes(buf[*counter..*counter + 8].try_into().unwrap());
        *counter += 8;
        ret
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
    fn deserialize(buf: &[u8], counter: &mut usize) -> Self {
        align_counter(counter, 4);
        let length = u32::from_le_bytes(buf[*counter..*counter + 4].try_into().unwrap()) as usize;
        *counter += 4;
        let mut ret = Vec::with_capacity(length);
        for _ in 0..length {
            let elem = T::deserialize(buf, counter);
            ret.push(elem);
        }
        ret
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
    fn deserialize(buf: &[u8], counter: &mut usize) -> Self {
        align_counter(counter, 1);

        let signature_length = buf[*counter] as usize;
        *counter += 1;

        let actual_signature =
            String::from_utf8(buf[*counter..*counter + signature_length].into()).unwrap();

        *counter += signature_length + 1; // +1 for null byte

        // the T itself will take care of padding
        let expected_signature = T::get_signature();
        if expected_signature != actual_signature {
            panic!(
                "expected signature {}, found {} instead",
                expected_signature, actual_signature
            );
        }
        let elem: T = T::deserialize(buf, counter);

        Self(elem)
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
    fn deserialize(_: &[u8], _: &mut usize) -> Self {
        panic!("we cannot really deref the dyn type to deserialize it");
        // align_counter(counter, 8);
        // let key = String::deserialize(buf, counter);
        // let val = DbusSerialize::deserialize(buf, counter);
        // Self {
        //     key,
        //     val: Box::new(val),
        // }
    }
}
