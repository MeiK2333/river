use super::error::{Error, Result};
use std::fs;
use std::io;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::tempfile;
use zip;

pub fn extract(filedir: &Path, body: &Vec<u8>) -> Result<()> {
    // 如果文件夹已存在，则删除再重新创建
    if filedir.exists() {
        try_io!(fs::remove_dir_all(&filedir));
    }
    try_io!(fs::create_dir(&filedir));
    let mut file = try_io!(tempfile());
    try_io!(file.write_all(&body));

    let mut archive = match zip::ZipArchive::new(file) {
        Ok(val) => val,
        Err(e) => return Err(Error::ZipError(e)),
    };
    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(val) => val,
            Err(e) => return Err(Error::ZipError(e)),
        };
        let outpath = match file.enclosed_name() {
            Some(path) => filedir.join(path).to_owned(),
            None => continue,
        };

        if (&*file.name()).ends_with('/') {
            try_io!(fs::create_dir_all(&outpath));
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    try_io!(fs::create_dir_all(&p));
                }
            }
            let mut outfile = try_io!(fs::File::create(&outpath));
            try_io!(io::copy(&mut file, &mut outfile));
        }

        if let Some(mode) = file.unix_mode() {
            try_io!(fs::set_permissions(
                &outpath,
                fs::Permissions::from_mode(mode)
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::process::Command;

    #[test]
    fn test1() {
        let filename = "hello.txt";
        let zipfile = "hello.zip";
        let prefix = "hello";
        let mut file = fs::File::create(&filename).unwrap();
        let _ = file.write_all(b"Hello World!").unwrap();
        let _ = Command::new("zip")
            .arg(&zipfile)
            .arg(&filename)
            .output()
            .expect("failed to execute process");
        // remove file
        fs::remove_file(&filename).unwrap();

        // extract
        let body = fs::read(&zipfile).unwrap();
        extract(&Path::new(&prefix.to_string()), &body).unwrap();
        fs::remove_file(&zipfile).unwrap();

        // check file
        assert!(Path::new(&prefix).join(&filename).exists());

        let body = fs::read_to_string(Path::new(&prefix).join(&filename)).unwrap();
        assert_eq!(body, "Hello World!");

        fs::remove_dir_all(&prefix).unwrap();
    }
}
