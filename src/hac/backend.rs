use anyhow::Result;
use std::{path::PathBuf, process::Command};
use tempdir::TempDir;

use crate::cache::Cache;

#[derive(Debug, Clone, Copy)]
pub enum Backend {
    Hacpack,
    Hactool,
}

impl Backend {
    pub fn path(&self) -> Result<PathBuf> {
        match self {
            Backend::Hacpack => {
                let hacpack = Cache::Hacpack;
                #[cfg(target_os = "windows")]
                {
                    return Ok(hacpack.path()?);
                }
                // if cfg!(target_os = "windows") {
                // } else {
                #[cfg(any(target_os = "linux", target_os = "android"))]
                {
                    if dbg!(hacpack.is_cached()) {
                        return Ok(hacpack.path()?);
                    } else {
                        return Ok(hacpack.from(make_hacpack()?)?.make_executable()?.path()?);
                    }
                }
            }
            Backend::Hactool => {
                let hactool = Cache::Hactool;
                #[cfg(target_os = "windows")]
                {
                    return Ok(hactool.path()?);
                }
                #[cfg(any(target_os = "linux", target_os = "android"))]
                {
                    if dbg!(hactool.is_cached()) {
                        return Ok(hactool.path()?);
                    } else {
                        return Ok(hactool.from(make_hactool()?)?.make_executable()?.path()?);
                    }
                }
            }
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn make_hacpack() -> Result<PathBuf> {
    use anyhow::bail;
    use std::fs;

    let src_dir = TempDir::new("hacpack")?.into_path();

    if !Command::new("git")
        .args(["clone", "https://github.com/The-4n/hacPack"])
        .arg(&src_dir)
        .status()?
        .success()
    {
        bail!("failed to clone the hacpack repo");
    };

    fs::rename(
        src_dir.join("config.mk.template"),
        src_dir.join("config.mk"),
    )?;

    if !Command::new("make")
        .current_dir(&src_dir)
        .status()?
        .success()
    {
        bail!("failed to build hacpack");
    }

    Ok(src_dir.join("hacpack"))
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn make_hactool() -> Result<PathBuf> {
    use anyhow::bail;
    use std::fs;

    let src_dir = TempDir::new("hactool")?.into_path();

    if !Command::new("git")
        .args(["clone", "https://github.com/SciresM/hactool"])
        .arg(&src_dir)
        .status()?
        .success()
    {
        bail!("failed to clone the hactool repo");
    };

    fs::rename(
        src_dir.join("config.mk.template"),
        src_dir.join("config.mk"),
    )?;

    if !Command::new("make")
        .current_dir(&src_dir)
        .status()?
        .success()
    {
        bail!("failed to build hactool");
    }

    Ok(src_dir.join("hactool"))
}
