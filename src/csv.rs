use std::{path::Path};

pub fn write_f32_csv<const N: usize>(
    path: impl AsRef<Path>,
    keys: [&str; N],
    valss: [&[f32]; N],
    header: &str,
) -> std::io::Result<()> {
    let mut string = header.to_string();
    string.push_str("\n\n");
    
    for s in keys {
        string.push_str(&format!("{}, ", s))
    }
    string.push_str("\n\n");

    for i in 0..N {
        for vals in valss {
            string.push_str(&format!("{:e}, ", vals[i]))
        }
        string.push('\n')
    }
    std::fs::write(path, string)
}



pub fn make_csv_header(comment: &str) -> String{
    let mut header = comment.to_string();
    header.push_str(&format!("File created at {:#?}", chrono::Utc::now().to_string()));
    header.push_str("\nThis file was generated by Speckmeter\nhttps://github.com/messiamax/speckmeter");
    header
}
