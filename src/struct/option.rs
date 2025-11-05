// use crate::{ZResult, ZStructEncode, ZWriter};

// impl<T: ZStructEncode> ZStructEncode for Option<T> {
//     fn z_len(&self) -> usize {
//         match self {
//             Some(value) => value.z_len(),
//             None => 0,
//         }
//     }

//     fn z_encode(&self, w: &mut ZWriter) -> ZResult<()> {
//         if let Some(value) = self {
//             value.z_encode(w)?;
//         }
//         Ok(())
//     }
// }
