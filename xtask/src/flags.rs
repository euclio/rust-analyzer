#![allow(unreachable_pub)]

use crate::install::{ClientOpt, Malloc, ServerOpt};

xflags::xflags! {
    src "./src/flags.rs"

    /// Run custom build command.
    cmd xtask {
        default cmd help {
            /// Print help information.
            optional -h, --help
        }

        /// Install rust-analyzer server or editor plugin.
        cmd install {
            /// Install only VS Code plugin.
            optional --client
            /// One of 'code', 'code-exploration', 'code-insiders', 'codium', or 'code-oss'.
            optional --code-bin name: String

            /// Install only the language server.
            optional --server
            /// Use mimalloc allocator for server
            optional --mimalloc
            /// Use jemalloc allocator for server
            optional --jemalloc
        }

        cmd codegen {
            optional --features
        }

        cmd lint {}
        cmd fuzz-tests {}
        cmd pre-cache {}

        cmd release {
            optional --dry-run
        }
        cmd promote {
            optional --dry-run
        }
        cmd dist {
            optional --nightly
            optional --client version: String
        }
        cmd metrics {
            optional --dry-run
        }
        /// Builds a benchmark version of rust-analyzer and puts it into `./target`.
        cmd bb
            required suffix: String
        {}
    }
}

// generated start
// The following code is generated by `xflags` macro.
// Run `env UPDATE_XFLAGS=1 cargo build` to regenerate.
#[derive(Debug)]
pub struct Xtask {
    pub subcommand: XtaskCmd,
}

#[derive(Debug)]
pub enum XtaskCmd {
    Help(Help),
    Install(Install),
    Codegen(Codegen),
    Lint(Lint),
    FuzzTests(FuzzTests),
    PreCache(PreCache),
    Release(Release),
    Promote(Promote),
    Dist(Dist),
    Metrics(Metrics),
    Bb(Bb),
}

#[derive(Debug)]
pub struct Help {
    pub help: bool,
}

#[derive(Debug)]
pub struct Install {
    pub client: bool,
    pub code_bin: Option<String>,
    pub server: bool,
    pub mimalloc: bool,
    pub jemalloc: bool,
}

#[derive(Debug)]
pub struct Codegen {
    pub features: bool,
}

#[derive(Debug)]
pub struct Lint;

#[derive(Debug)]
pub struct FuzzTests;

#[derive(Debug)]
pub struct PreCache;

#[derive(Debug)]
pub struct Release {
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct Promote {
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct Dist {
    pub nightly: bool,
    pub client: Option<String>,
}

#[derive(Debug)]
pub struct Metrics {
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct Bb {
    pub suffix: String,
}

impl Xtask {
    pub const HELP: &'static str = Self::HELP_;

    pub fn from_env() -> xflags::Result<Self> {
        Self::from_env_()
    }
}
// generated end

impl Install {
    pub(crate) fn server(&self) -> Option<ServerOpt> {
        if self.client && !self.server {
            return None;
        }
        let malloc = if self.mimalloc {
            Malloc::Mimalloc
        } else if self.jemalloc {
            Malloc::Jemalloc
        } else {
            Malloc::System
        };
        Some(ServerOpt { malloc })
    }
    pub(crate) fn client(&self) -> Option<ClientOpt> {
        if !self.client && self.server {
            return None;
        }
        Some(ClientOpt { code_bin: self.code_bin.clone() })
    }
}
