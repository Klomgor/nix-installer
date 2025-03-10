use std::str::FromStr;

use crate::settings::UrlOrPath;

#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Distribution {
    Nix,
    DeterminateNix,
}

impl Distribution {
    pub fn tarball_location_or(&self, user_preference: &Option<UrlOrPath>) -> TarballLocation {
        if let Some(pref) = user_preference {
            return TarballLocation::UrlOrPath(pref.clone());
        }

        self.tarball_location()
    }

    pub fn tarball_location(&self) -> TarballLocation {
        match self {
            Distribution::Nix => TarballLocation::UrlOrPath(
                UrlOrPath::from_str(NIX_TARBALL_URL)
                    .expect("Fault: the built-in Nix tarball URL does not parse."),
            ),
            Distribution::DeterminateNix => {
                TarballLocation::InMemory(
                    DETERMINATE_NIX_TARBALL_PATH.expect("Fault: this build of Determinate Nix Installer is not equipped to install Determinate Nix."),
                    DETERMINATE_NIX_TARBALL.expect("Fault: this build of Determinate Nix Installer is not equipped to install Determinate Nix.")
                )
            },
        }
    }
}

pub enum TarballLocation {
    UrlOrPath(UrlOrPath),
    InMemory(&'static str, &'static [u8]),
}

pub const NIX_TARBALL_URL: &str = env!("NIX_TARBALL_URL");

#[cfg(feature = "determinate-nix")]
pub const DETERMINATE_NIX_TARBALL_PATH: Option<&str> = Some(env!("DETERMINATE_NIX_TARBALL_PATH"));
#[cfg(feature = "determinate-nix")]
/// The DETERMINATE_NIX_TARBALL environment variable should point to a target-appropriate
/// Determinate Nix installation tarball, like determinate-nix-2.21.2-aarch64-darwin.tar.xz.
/// The contents are embedded in the resulting binary.
pub const DETERMINATE_NIX_TARBALL: Option<&[u8]> =
    Some(include_bytes!(env!("DETERMINATE_NIX_TARBALL_PATH")));

#[cfg(feature = "determinate-nix")]
/// The DETERMINATE_NIXD_BINARY_PATH environment variable should point to a target-appropriate
/// static build of the Determinate Nixd binary. The contents are embedded in the resulting
/// binary if the determinate-nix feature is turned on.
pub const DETERMINATE_NIXD_BINARY: Option<&[u8]> =
    Some(include_bytes!(env!("DETERMINATE_NIXD_BINARY_PATH")));

#[cfg(not(feature = "determinate-nix"))]
pub const DETERMINATE_NIXD_BINARY: Option<&[u8]> = None;
#[cfg(not(feature = "determinate-nix"))]
pub const DETERMINATE_NIX_TARBALL: Option<&[u8]> = None;
#[cfg(not(feature = "determinate-nix"))]
pub const DETERMINATE_NIX_TARBALL_PATH: Option<&str> = None;
