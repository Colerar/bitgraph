#![allow(dead_code)]

use core::str;
use std::{
  borrow::Borrow,
  env,
  ffi::OsStr,
  io::BufReader,
  path::{Path, PathBuf},
  process::{Command, Stdio},
  str::FromStr,
};

use crate::serde_ext::*;

use anyhow::{ensure, Context};
use serde::Deserialize;

const WHERE_PROG: &str = if cfg!(windows) { "where.exe" } else { "which" };

fn work_dir() -> PathBuf {
  env::current_dir().expect("Working dir is not found")
}

fn find_program(name: &str) -> Option<PathBuf> {
  let exe_file = env::current_exe().expect("Executable file is not found");
  let exe_dir = exe_file
    .parent()
    .expect("The parent of executable file is not found");
  let exe_dir_lib = exe_dir.join("lib");
  let work_dir = work_dir();
  let mut work_dir_lib = work_dir.clone();
  work_dir_lib.push("lib");
  let usr_local_bin = PathBuf::from_str("/usr/local/bin").unwrap();
  let usr_bin = PathBuf::from_str("/usr/bin").unwrap();
  let dirs: Vec<&Path> = vec![
    exe_dir,
    exe_dir_lib.as_path(),
    work_dir.as_path(),
    work_dir_lib.as_path(),
    usr_local_bin.as_path(),
    usr_bin.as_path(),
  ];

  dirs
    .into_iter()
    .find_map(|i| {
      let exe = if cfg!(windows) {
        i.join(format!("{name}.exe"))
      } else {
        i.join(name)
      };
      if exe.exists() {
        Some(exe)
      } else {
        None
      }
    })
    .or_else(|| {
      let where_result = Command::new(WHERE_PROG)
        .arg(name)
        .stderr(Stdio::inherit())
        .output();
      if let Ok(output) = where_result {
        let path = String::from_utf8_lossy(&output.stdout).to_string();
        let path = path.trim_end();
        let buf = PathBuf::from(path.to_string());
        if !buf.exists() {
          return None;
        }
        Some(buf)
      } else {
        None
      }
    })
}

pub struct FfProbe {
  path: String,
}

impl FfProbe {
  pub fn find() -> anyhow::Result<FfProbe> {
    let path = find_program("ffprobe").context("Failed to find ffprobe")?;
    let path = path.to_string_lossy().to_string();

    Ok(FfProbe { path })
  }

  pub fn execute_json<T, I, S>(&self, args: I) -> anyhow::Result<T>
  where
    T: serde::de::DeserializeOwned,
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
  {
    let output = Command::new(self.path.as_str())
      .args(args)
      .stderr(Stdio::inherit())
      .stdout(Stdio::piped())
      .spawn()
      .context("Failed to spawn ffprobe")?;
    let child_stdout = output.stdout.unwrap();
    let reader = BufReader::new(child_stdout);
    let value: T = serde_json::from_reader(reader)
      .context(concat!("Failed to deserialize type ", stringify!(T)))?;
    Ok(value)
  }

  pub fn fetch_version(&self) -> anyhow::Result<ProgramVersion> {
    let result: ProgramVersionWrap = self
      .execute_json(["-hide_banner", "-show_program_version", "-of", "json=c=1"])
      .context("Failed to fetch version")?;
    Ok(result.program_version)
  }

  pub fn probe<P>(&self, path: P, stream_select: Option<&str>) -> anyhow::Result<ProbeData>
  where
    P: AsRef<Path>,
  {
    let path = path.as_ref();
    ensure!(
      path.exists(),
      "Path `{}` does not exist",
      path.to_string_lossy()
    );
    ensure!(
      path.is_file(),
      "Path `{}` is not a file",
      path.to_string_lossy()
    );
    let path = path.to_string_lossy();
    let mut args = vec![
      "-hide_banner",
      "-show_error",
      "-show_entries",
      "packet=stream_index,pts,pts_time,size:stream:format",
      "-of",
      "json=c=1",
      path.borrow(),
    ];
    if let Some(stream_select) = stream_select {
      args.push("-select_streams");
      args.push(stream_select);
    }
    let data: ProbeData = self
      .execute_json(args)
      .context("Failed to get probe info")?;
    Ok(data)
  }
}

#[derive(Deserialize, Debug)]
pub struct ProbeData {
  pub packets: Option<Vec<Packet>>,
  pub streams: Option<Vec<Stream>>,
  pub format: Option<Format>,
  pub error: Option<Error>,
}

#[derive(Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Packet {
  pub stream_index: u64,
  pub pts: i64,
  #[serde(deserialize_with = "de_from_string")]
  pub pts_time: f64,
  #[serde(deserialize_with = "de_from_string")]
  pub size: u32,
}

#[derive(Deserialize, Debug)]
pub struct Stream {
  pub index: u64,
  pub codec_name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Format {
  pub filename: String,
  pub format_name: String,
  pub format_long_name: String,
  #[serde(deserialize_with = "na_or_from_string")]
  pub start_time: Option<f64>,
  #[serde(deserialize_with = "na_or_from_string")]
  pub duration: Option<f64>,
  #[serde(deserialize_with = "de_from_string")]
  pub size: u64,
  #[serde(deserialize_with = "de_from_string")]
  pub bit_rate: u64,
}

#[derive(Deserialize, Debug)]
pub struct Error {
  pub code: i32,
  #[serde(rename = "string")]
  pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct ProgramVersionWrap {
  pub program_version: ProgramVersion,
}

#[derive(Deserialize, Debug)]
pub struct ProgramVersion {
  pub version: String,
  pub copyright: String,
  pub compiler_ident: String,
  pub configuration: String,
}

#[cfg(test)]
mod tests {
  use super::*;
  use indoc::indoc;

  #[test]
  fn deser_packet() {
    const PACKET_JSON: &str = indoc! {r#"
      {
        "stream_index": 0,
        "pts": 114514,
        "pts_time": "123.123",
        "size": "1"
      }
    "#};
    let result: serde_json::Result<Packet> = serde_json::from_str(PACKET_JSON);
    assert!(result.is_ok());
    assert_eq!(
      result.unwrap(),
      Packet {
        stream_index: 0,
        pts: 114514,
        pts_time: 123.123,
        size: 1,
      }
    );
  }
}
