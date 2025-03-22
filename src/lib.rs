use std::fs;

use zed::Command;
use zed_extension_api as zed;

struct Moxide {
    cached_binary_path: Option<String>,
}

impl Moxide {
    fn language_server_binary_path(
        &mut self,
        config: zed::LanguageServerConfig,
        worktree: &zed::Worktree,
    ) -> zed::Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        // first check if already installed to path
        if let Some(path) = worktree.which("markdown-oxide") {
            self.cached_binary_path = Some(path.clone());
            return Ok(path);
        }

        zed::set_language_server_installation_status(
            &config.name,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            "feel-ix-343/markdown-oxide",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let (asset_name, asset_extension, lsp_extension) = (
            format!(
                "markdown-oxide-{version}-{arch}-{os}",
                version = release.version,
                arch = match arch {
                    zed::Architecture::Aarch64 => "aarch64",
                    zed::Architecture::X86 => return Err(format!("unsupported platform")),
                    zed::Architecture::X8664 => "x86_64",
                },
                os = match platform {
                    zed::Os::Mac => "apple-darwin",
                    zed::Os::Linux => "unknown-linux-gnu",
                    zed::Os::Windows => "pc-windows-gnu",
                }
            ),
            match platform {
                zed::Os::Mac | zed::Os::Linux => ".tar.gz",
                zed::Os::Windows => ".zip",
            },
            match platform {
                zed::Os::Windows => ".exe",
                _ => "",
            },
        );

        let asset_full_name = format!("{}{}", asset_name, asset_extension);
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_full_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_full_name))?;

        let version_dir = format!("markdown-oxide-{}", release.version);
        let binary_path = format!("{version_dir}/{asset_name}/markdown-oxide{lsp_extension}");

        if !fs::metadata(&binary_path).map_or(false, |stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                &config.name,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                match platform {
                    zed::Os::Mac | zed::Os::Linux => zed::DownloadedFileType::GzipTar,
                    zed::Os::Windows => zed::DownloadedFileType::Zip,
                },
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;

            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(&entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        return Ok(binary_path);
    }
}

impl zed::Extension for Moxide {
    fn language_server_command(
        &mut self,
        config: zed::LanguageServerConfig,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let binary_path = self.language_server_binary_path(config, worktree)?;

        return Ok(Command {
            command: binary_path,
            args: Default::default(),
            env: Default::default(),
        });
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        Moxide {
            cached_binary_path: None,
        }
    }
}

zed::register_extension!(Moxide);
