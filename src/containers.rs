// impl<T: DbusSerialize> DbusSerialize for Variant<T> {
//     fn get_signature() -> String {
//         "v".to_string()
//     }
//     fn serialize(&self, buf: &mut Vec<u8>) {
//         // no alignment needed, as variant is 1-align
//         let sub_type = T::get_signature();
//         let signature_length = sub_type.len() as u8; // signature length must be < 256
//         buf.push(signature_length);
//         buf.extend_from_slice(sub_type.as_bytes());
//         buf.push(0);
//         self.0.serialize(buf);
//     }
// }
// impl DbusSerialize for Structure {
//     fn get_signature() -> String {
//         "(sv)".to_string()
//     }
//     fn serialize(&self, buf: &mut Vec<u8>) {

//     }
// }
