use zed::{Command, LanguageServerInstallationStatus};
use zed_extension_api as zed;

struct Moxide {

}


impl zed::Extension for Moxide {
    fn language_server_command(
            &mut self,
            config: zed::LanguageServerConfig,
            worktree: &zed::Worktree,
        ) -> zed::Result<zed::Command> {

        let path = worktree
            .which("markdown-oxide")
            .ok_or_else(|| "markdown-oxide is not installed; check repo to install".to_string())?;

        Ok(Command {
            command: path,
            args: Default::default(),
            env: Default::default()
        })

    }

    fn new() -> Self
        where
            Self: Sized {
        
        Moxide{}
    }
}

zed::register_extension!(Moxide);
