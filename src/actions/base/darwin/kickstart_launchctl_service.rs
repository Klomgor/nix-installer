use serde::Serialize;
use tokio::process::Command;

use crate::execute_command;

use crate::actions::{Action, ActionDescription, ActionState, Actionable};

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct KickstartLaunchctlService {
    unit: String,
    action_state: ActionState,
}

impl KickstartLaunchctlService {
    #[tracing::instrument(skip_all)]
    pub async fn plan(unit: String) -> Result<Self, KickstartLaunchctlServiceError> {
        Ok(Self {
            unit,
            action_state: ActionState::Uncompleted,
        })
    }
}

#[async_trait::async_trait]
impl Actionable for KickstartLaunchctlService {
    type Error = KickstartLaunchctlServiceError;

    fn describe_execute(&self) -> Vec<ActionDescription> {
        let Self { unit, action_state } = self;
        if *action_state == ActionState::Completed {
            vec![]
        } else {
            vec![ActionDescription::new(
                format!("Kickstart the launchctl unit `{unit}`"),
                vec![
                    "The `nix` command line tool communicates with a running Nix daemon managed by your init system".to_string()
                ]
            )]
        }
    }

    #[tracing::instrument(skip_all, fields(
        unit = %self.unit,
    ))]
    async fn execute(&mut self) -> Result<(), Self::Error> {
        let Self { unit, action_state } = self;
        if *action_state == ActionState::Completed {
            tracing::trace!("Already completed: Kickstarting launchctl unit");
            return Ok(());
        }
        tracing::debug!("Kickstarting launchctl unit");

        execute_command(
            Command::new("launchctl")
                .arg("kickstart")
                .arg("-k")
                .arg(unit),
        )
        .await
        .map_err(KickstartLaunchctlServiceError::Command)?;

        tracing::trace!("Kickstarted launchctl unit");
        *action_state = ActionState::Completed;
        Ok(())
    }

    fn describe_revert(&self) -> Vec<ActionDescription> {
        if self.action_state == ActionState::Uncompleted {
            vec![]
        } else {
            vec![ActionDescription::new(
                "Kick".to_string(),
                vec![
                    "The `nix` command line tool communicates with a running Nix daemon managed by your init system".to_string()
                ]
            )]
        }
    }

    #[tracing::instrument(skip_all, fields(
        unit = %self.unit,
    ))]
    async fn revert(&mut self) -> Result<(), Self::Error> {
        let Self { unit, action_state } = self;
        if *action_state == ActionState::Uncompleted {
            tracing::trace!("Already reverted: Stopping launchctl unit");
            return Ok(());
        }
        tracing::debug!("Stopping launchctl unit");

        execute_command(Command::new("launchctl").arg("stop").arg(unit))
            .await
            .map_err(KickstartLaunchctlServiceError::Command)?;

        tracing::trace!("Stopped launchctl unit");
        *action_state = ActionState::Completed;
        Ok(())
    }
}

impl From<KickstartLaunchctlService> for Action {
    fn from(v: KickstartLaunchctlService) -> Self {
        Action::DarwinKickStartLaunchctlService(v)
    }
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum KickstartLaunchctlServiceError {
    #[error("Failed to execute command")]
    Command(
        #[source]
        #[serde(serialize_with = "crate::serialize_error_to_display")]
        std::io::Error,
    ),
}