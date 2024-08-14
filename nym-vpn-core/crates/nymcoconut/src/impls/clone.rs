// use crate::{
//     BlindedSignature, Bytable,
// };
//
// macro_rules! impl_clone {
//     ($struct:ident) => {
//         impl Clone for $struct {
//             fn clone(&self) -> Self {
//                 Self::try_from_byte_slice(&self.to_byte_vec()).unwrap()
//             }
//         }
//     };
// }

// impl_clone!(BlindedSignature);
