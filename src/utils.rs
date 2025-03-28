// use async_std::{fs::File, io::ReadExt, path::Path};

// pub async fn is_plain_text_by_content(path: &Path) -> bool {
//     let mut buffer = [0; 512]; // Read up to 512 bytes
//     let Ok(mut file) = File::open(path).await else {
//         return false;
//     };
//     let Ok(bytes_read) = file.read(&mut buffer).await else {
//         return false;
//     };

//     String::from_utf8(buffer[..bytes_read].to_vec())
//         .map(|s| s.chars().all(|c| c.is_ascii() && !c.is_ascii_control()))
//         .unwrap_or(false)
// }
