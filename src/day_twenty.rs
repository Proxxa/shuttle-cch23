use std::{io::{Cursor, Read}, process::Command, fs::File, path::PathBuf, ffi::OsStr};

use log::private::error;
use rocket::{http::Status, serde::json::Json, *};
use tempfile::tempdir;

fn map_err<const CODE: u16>(to_string: impl ToString) -> (Status, String) {
    (Status { code: CODE }, dbg!(to_string.to_string()))
}

#[post("/archive_files", data = "<data>")]
pub fn archive_files(data: Vec<u8>) -> Result<Json<usize>, (Status, String)> {
    tar::Archive::new(Cursor::new(data))
        .entries()
        .map_err(map_err::<500>)
        .map(|e| Json(e.count()))
}

#[post("/archive_files_size", data = "<data>")]
pub fn archive_files_size(
    data: Vec<u8>,
) -> Result<Json<u64>, (Status, String)> {
    tar::Archive::new(Cursor::new(data))
        .entries()
        .and_then(|mut e| e.try_fold(0u64, |size, b| b.map(|b| size + b.size())))
        .map(|size| Json(size))
        .map_err(map_err::<500>)
}

#[post("/cookie", data = "<data>")]
pub async fn cookie(data: Vec<u8>) -> Result<String, (Status, String)> {
    let tempdir = tempdir().map_err(map_err::<500>)?;
    tar::Archive::new(Cursor::new(data))
        .unpack(tempdir.path())
        .map_err(map_err::<500>)
        .and_then(|_| {
            error!("Got to the big scary block");

            let output = Command::new("git")
                .args(["log", "--format=%cn,%H", "christmas",/* "--", "santa.txt"*/])
                .current_dir(tempdir.path())
                .output()
                .map_err(map_err::<500>)?;
            let output = String::from_utf8(output.stdout).unwrap();
            info!("Output:\n\n{output}\n\n");

            let output = Command::new("git")
                .args(["log", "--format=%cn,%H", "christmas", /*"--", "santa.txt"*/])
                .current_dir(tempdir.path())
                .output()
                .map_err(map_err::<500>)?;
            
            // The only error here should occur if `christmas` is not a real branch. (:
            if !output.status.success() {
                error!("Returning 400!!");
                let err_string = String::from_utf8(output.stderr).map_err(map_err::<500>)?;
                // This is dumb. I like it.
                return dbg!(Err(err_string).map_err(map_err::<400>));
            }


            // This isn't a good idea.
            // I'm doing it anyway. I will regret it.
            for (author, commit) in String::from_utf8(output.stdout).map_err(map_err::<500>)?.lines().map(|l| l.split_once(",").unwrap()) {

                let output = Command::new("git")
                    .args(["checkout", commit, "--force"])
                    .current_dir(tempdir.path())
                    .output()
                    .map_err(map_err::<500>)?;

                // If this fails, I've failed horribly somewhere.
                let fail_string = String::from_utf8(output.stderr).map_err(map_err::<500>)?;
                assert!(output.status.success(), "This is a very big problem.\n{fail_string}\n");



                let mut checstr = String::new();
                let Ok(path) = find_santa(tempdir.path().to_owned()) else { continue };

                if !path.exists() { 
                    error!("Doesn't exist for {author} {commit} despite getting to this point");
                    continue
                }

                File::open(path).map_err(map_err::<501>)?.read_to_string(&mut checstr).map_err(map_err::<502>)?;

                error!("{author} {commit}\n{checstr}");

                if checstr.contains("COOKIE") {
                    return dbg!(Ok(format!("{author} {commit}")));
                }
            }
            
            Err(dbg!("Could not find santa's cookie.")).map_err(map_err::<400>)
        })
        .map(|a| { drop(tempdir); dbg!(a) })
        .map_err(|e| dbg!(e))
}


fn find_santa(pat: PathBuf) -> Result<PathBuf, ()> {

    if pat.is_file() {
        info!("Hit file: {}", pat.display())
    }
    if pat.is_file() && pat.file_name().expect("Dear god why does this not have a name") == OsStr::new("santa.txt") {
        return Ok(pat);
    }

    if pat.is_dir() {
        for e in std::fs::read_dir(pat).expect("I can't be bothered to do this ") {
            let Ok(e) = e else { continue };

            match find_santa(e.path()) {
                ok @ Ok(_) => return ok,
                _ => continue,
            }
        }
    }
    
    Err(())
}