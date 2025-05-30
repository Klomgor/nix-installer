use std::path::{Path, PathBuf};

use tracing::{span, Span};

use crate::action::{ActionError, ActionErrorKind, ActionTag, StatefulAction};

use crate::action::common::configure_init_service::{SocketFile, UnitSrc};
use crate::action::{common::ConfigureInitService, Action, ActionDescription};
use crate::settings::InitSystem;
use crate::util::OnMissing;

// Linux
const SERVICE_SRC: &str = "/nix/var/nix/profiles/default/lib/systemd/system/nix-daemon.service";
const SERVICE_DEST: &str = "/etc/systemd/system/nix-daemon.service";

// Darwin
const DARWIN_NIX_DAEMON_SOURCE: &str =
    "/nix/var/nix/profiles/default/Library/LaunchDaemons/org.nixos.nix-daemon.plist";
pub(crate) const DARWIN_NIX_DAEMON_DEST: &str = "/Library/LaunchDaemons/org.nixos.nix-daemon.plist";
const DARWIN_LAUNCHD_SERVICE_NAME: &str = "org.nixos.nix-daemon";

/**
Configure the init to run the Nix daemon
*/
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(tag = "action_name", rename = "create_upstream_init_service")]
pub struct ConfigureUpstreamInitService {
    configure_init_service: StatefulAction<ConfigureInitService>,
}

impl ConfigureUpstreamInitService {
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn plan(
        init: InitSystem,
        start_daemon: bool,
    ) -> Result<StatefulAction<Self>, ActionError> {
        let service_src: Option<UnitSrc> = match init {
            InitSystem::Launchd => Some(UnitSrc::Path(DARWIN_NIX_DAEMON_SOURCE.into())),
            InitSystem::Systemd => Some(UnitSrc::Path(SERVICE_SRC.into())),
            InitSystem::None => None,
        };
        let service_dest: Option<PathBuf> = match init {
            InitSystem::Launchd => {
                // NOTE(cole-h): if the determinate daemon exists and we're installing the upstream
                // daemon, we need to remove the old daemon unit -- we used to have a bug[1] where
                // these service files wouldn't get removed, so we can't rely on them not being
                // there after phase 1 of the uninstall
                // [1]: https://github.com/DeterminateSystems/nix-installer/pull/1266
                crate::util::remove_file(
                    Path::new(
                        super::configure_determinate_nixd_init_service::DARWIN_NIXD_DAEMON_DEST,
                    ),
                    OnMissing::Ignore,
                )
                .await
                .map_err(|e| {
                    Self::error(ActionErrorKind::Remove(
                        super::configure_determinate_nixd_init_service::DARWIN_NIXD_DAEMON_DEST
                            .into(),
                        e,
                    ))
                })?;

                Some(DARWIN_NIX_DAEMON_DEST.into())
            },
            InitSystem::Systemd => Some(SERVICE_DEST.into()),
            InitSystem::None => None,
        };
        let service_name: Option<String> = match init {
            InitSystem::Launchd => Some(DARWIN_LAUNCHD_SERVICE_NAME.into()),
            _ => None,
        };

        let configure_init_service = ConfigureInitService::plan(
            init,
            start_daemon,
            service_src,
            service_dest,
            service_name,
            vec![SocketFile {
                name: "nix-daemon.socket".into(),
                src: UnitSrc::Path(
                    "/nix/var/nix/profiles/default/lib/systemd/system/nix-daemon.socket".into(),
                ),
                dest: "/etc/systemd/system/nix-daemon.socket".into(),
            }],
        )
        .await
        .map_err(Self::error)?;

        Ok(Self {
            configure_init_service,
        }
        .into())
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "create_upstream_init_service")]
impl Action for ConfigureUpstreamInitService {
    fn action_tag() -> ActionTag {
        ActionTag("create_upstream_init_service")
    }
    fn tracing_synopsis(&self) -> String {
        "Configure upstream Nix daemon service".to_string()
    }

    fn tracing_span(&self) -> Span {
        span!(tracing::Level::DEBUG, "create_upstream_init_service",)
    }

    fn execute_description(&self) -> Vec<ActionDescription> {
        vec![ActionDescription::new(
            self.tracing_synopsis(),
            vec![self.configure_init_service.tracing_synopsis()],
        )]
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn execute(&mut self) -> Result<(), ActionError> {
        self.configure_init_service
            .try_execute()
            .await
            .map_err(Self::error)?;

        Ok(())
    }

    fn revert_description(&self) -> Vec<ActionDescription> {
        vec![ActionDescription::new(
            "Remove upstream Nix daemon service".to_string(),
            vec![self.configure_init_service.tracing_synopsis()],
        )]
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn revert(&mut self) -> Result<(), ActionError> {
        self.configure_init_service.try_revert().await?;

        Ok(())
    }
}
