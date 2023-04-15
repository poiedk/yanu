pub mod repack;
pub mod unpack;
pub mod update;

use crate::vfs::{nacp::NacpData, ticket::TitleKey};
use common::{defines::DEFAULT_TITLEKEYS_PATH, utils::move_file};
use eyre::{bail, eyre, Result};
use fs_err as fs;
use std::{
    io::{self, ErrorKind},
    path::PathBuf,
};
use tracing::{info, warn};

pub fn clear_titlekeys() -> Result<()> {
    match fs::remove_file(DEFAULT_TITLEKEYS_PATH.as_path()) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => {
            bail!(err)
        }
    }
}

/// Store TitleKeys to `DEFAULT_TITLEKEYS_PATH`.
pub fn store_titlekeys<'a, I>(keys: I) -> Result<()>
where
    I: Iterator<Item = &'a TitleKey>,
{
    info!(keyfile = ?DEFAULT_TITLEKEYS_PATH.as_path(), "Storing TitleKeys");
    fs::create_dir_all(DEFAULT_TITLEKEYS_PATH.parent().unwrap())?;
    fs::write(
        DEFAULT_TITLEKEYS_PATH.as_path(),
        keys.map(|key| key.to_string())
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .map_err(|err| eyre!(err))
}

#[derive(Debug, Default, Clone)]
pub struct CleanupDirsOnDrop {
    dirs: Vec<PathBuf>,
}

impl CleanupDirsOnDrop {
    pub fn new<I: IntoIterator<Item = PathBuf>>(dirs: I) -> Self {
        Self {
            dirs: dirs.into_iter().collect(),
        }
    }
    fn close_impl(&mut self) -> Result<()> {
        // TODO: look up how to propogate multiple errors
        let mut outerr = None;
        for dir in &self.dirs {
            match fs::remove_dir_all(dir) {
                Ok(_) => {}
                Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                Err(err) => {
                    warn!(%err);
                    outerr.get_or_insert(err);
                }
            }
        }

        if let Some(err) = outerr {
            bail!(err)
        }
        Ok(())
    }
    pub fn close(mut self) -> Result<()> {
        let res = self.close_impl();
        std::mem::forget(self);
        res
    }
}

impl Drop for CleanupDirsOnDrop {
    fn drop(&mut self) {
        _ = self.close_impl();
    }
}

macro_rules! hacpack_cleanup_install {
    ($parent:expr) => {
        crate::utils::CleanupDirsOnDrop::new([
            $parent.join("hacpack_temp"),
            $parent.join("hacpack_backup"),
        ])
    };
}

pub(super) use hacpack_cleanup_install;

pub fn custom_nsp_rename(
    nsp_path: &mut PathBuf,
    nacp_data: &NacpData,
    program_id: &str,
    suffix: &str,
) -> Result<()> {
    let dest = nsp_path
        .parent()
        .ok_or_else(|| eyre!("Failed to get parent"))?
        .join(format!(
            "{} [{}][v{}]{suffix}.nsp",
            dbg!(nacp_data.get_application_name()),
            program_id,
            dbg!(nacp_data.get_application_version())
        ));

    info!(from = %nsp_path.display(), to = %dest.display(), "Moving");
    move_file(&nsp_path, &dest)?;
    *nsp_path = dest;

    Ok(())
}
