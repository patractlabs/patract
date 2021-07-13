use crate::chain_spec;
use sc_cli;
use std::path::PathBuf;
use structopt::StructOpt;
use sc_telemetry::TelemetryEndpoints;
use sc_service::config::PrometheusConfig;
use sc_service::{TransactionPoolOptions, BasePath};
use std::net::SocketAddr;

#[derive(Debug, StructOpt)]
pub enum Subcommand {
    /// Key management cli utilities
    Key(sc_cli::KeySubcommand),

    /// Export the genesis state of the parachain.
    #[structopt(name = "export-genesis-state")]
    ExportGenesisState(ExportGenesisStateCommand),

    /// Export the genesis wasm of the parachain.
    #[structopt(name = "export-genesis-wasm")]
    ExportGenesisWasm(ExportGenesisWasmCommand),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(cumulus_client_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),
}

/// Command for exporting the genesis state of the parachain
#[derive(Debug, StructOpt)]
pub struct ExportGenesisStateCommand {
    /// Output file name or stdout if unspecified.
    #[structopt(parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Id of the parachain this state is for.
    #[structopt(long, default_value = "100")]
    pub parachain_id: u32,

    /// Write output in binary. Default is to write in hex.
    #[structopt(short, long)]
    pub raw: bool,

    /// The name of the chain for that the genesis state should be exported.
    #[structopt(long)]
    pub chain: Option<String>,
}

/// Command for exporting the genesis wasm file.
#[derive(Debug, StructOpt)]
pub struct ExportGenesisWasmCommand {
    /// Output file name or stdout if unspecified.
    #[structopt(parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Write output in binary. Default is to write in hex.
    #[structopt(short, long)]
    pub raw: bool,

    /// The name of the chain for that the genesis wasm file should be exported.
    #[structopt(long)]
    pub chain: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct RunCmd {
    #[structopt(flatten)]
    pub base: sc_cli::RunCmd,

    /// Id of the parachain this collator collates for.
    #[structopt(long)]
    pub parachain_id: Option<u32>,

    /// Run node as collator.
	///
	/// Note that this is the same as running with `--validator`.
    #[structopt(long, conflicts_with = "validator")]
    pub collator: bool,
}

impl std::ops::Deref for RunCmd {
    type Target = sc_cli::RunCmd;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

/// A non-redundant version of the `RunCmd` that sets the `validator` field when the
/// original `RunCmd` had the `colaltor` field.
/// This is how we make `--collator` imply `--validator`.
pub struct NormalizedRunCmd {
    /// The cumulus RunCmd inherents from sc_cli's
    pub base: sc_cli::RunCmd,
    /// Id of the parachain this collator collates for.
    pub parachain_id: Option<u32>,
}

impl RunCmd {
    /// Create a [`NormalizedRunCmd`] which merges the `collator` cli argument into `validator` to have only one.
    pub fn normalize(&self) -> NormalizedRunCmd {
        let mut new_base = self.base.clone();

        new_base.validator = self.base.validator || self.collator;

        NormalizedRunCmd {
            base: new_base,
            parachain_id: self.parachain_id,
        }
    }
}

impl sc_cli::CliConfiguration for NormalizedRunCmd {
    fn shared_params(&self) -> &sc_cli::SharedParams {
        self.base.shared_params()
    }

    fn import_params(&self) -> Option<&sc_cli::ImportParams> {
        self.base.import_params()
    }

    fn network_params(&self) -> Option<&sc_cli::NetworkParams> {
        self.base.network_params()
    }

    fn keystore_params(&self) -> Option<&sc_cli::KeystoreParams> {
        self.base.keystore_params()
    }

    fn offchain_worker_params(&self) -> Option<&sc_cli::OffchainWorkerParams> {
        self.base.offchain_worker_params()
    }

    fn node_name(&self) -> sc_cli::Result<String> {
        self.base.node_name()
    }

    fn dev_key_seed(&self, is_dev: bool) -> sc_cli::Result<Option<String>> {
        self.base.dev_key_seed(is_dev)
    }

    fn telemetry_endpoints(
        &self,
        chain_spec: &Box<dyn sc_cli::ChainSpec>,
    ) -> sc_cli::Result<Option<TelemetryEndpoints>> {
        self.base.telemetry_endpoints(chain_spec)
    }

    fn role(&self, is_dev: bool) -> sc_cli::Result<sc_cli::Role> {
        self.base.role(is_dev)
    }

    fn force_authoring(&self) -> sc_cli::Result<bool> {
        self.base.force_authoring()
    }

    fn prometheus_config(&self, default_listen_port: u16) -> sc_cli::Result<Option<PrometheusConfig>> {
        self.base.prometheus_config(default_listen_port)
    }

    fn disable_grandpa(&self) -> sc_cli::Result<bool> {
        self.base.disable_grandpa()
    }

    fn rpc_ws_max_connections(&self) -> sc_cli::Result<Option<usize>> {
        self.base.rpc_ws_max_connections()
    }

    fn rpc_cors(&self, is_dev: bool) -> sc_cli::Result<Option<Vec<String>>> {
        self.base.rpc_cors(is_dev)
    }

    fn rpc_http(&self, default_listen_port: u16) -> sc_cli::Result<Option<SocketAddr>> {
        self.base.rpc_http(default_listen_port)
    }

    fn rpc_ipc(&self) -> sc_cli::Result<Option<String>> {
        self.base.rpc_ipc()
    }

    fn rpc_ws(&self, default_listen_port: u16) -> sc_cli::Result<Option<SocketAddr>> {
        self.base.rpc_ws(default_listen_port)
    }

    fn rpc_methods(&self) -> sc_cli::Result<sc_service::config::RpcMethods> {
        self.base.rpc_methods()
    }

    fn transaction_pool(&self) -> sc_cli::Result<TransactionPoolOptions> {
        self.base.transaction_pool()
    }

    fn max_runtime_instances(&self) -> sc_cli::Result<Option<usize>> {
        self.base.max_runtime_instances()
    }

    fn base_path(&self) -> sc_cli::Result<Option<BasePath>> {
        self.base.base_path()
    }
}


#[derive(Debug, StructOpt)]
#[structopt(settings = &[
structopt::clap::AppSettings::GlobalVersion,
structopt::clap::AppSettings::ArgsNegateSubcommands,
structopt::clap::AppSettings::SubcommandsNegateReqs,
])]
pub struct Cli {
    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[structopt(flatten)]
    pub run: RunCmd,

    /// Run node as collator.
    ///
    /// Note that this is the same as running with `--validator`.
    // #[structopt(long, conflicts_with = "validator")]
    // pub collator: bool,

    /// Relaychain arguments
    #[structopt(raw = true)]
    pub relaychain_args: Vec<String>,
}

#[derive(Debug)]
pub struct RelayChainCli {
    /// The actual relay chain cli object.
    pub base: polkadot_cli::RunCmd,

    /// Optional chain id that should be passed to the relay chain.
    pub chain_id: Option<String>,

    /// The base path that should be used by the relay chain.
    pub base_path: Option<PathBuf>,
}

impl RelayChainCli {
    /// Parse the relay chain CLI parameters using the para chain `Configuration`.
    pub fn new<'a>(
        para_config: &sc_service::Configuration,
        relay_chain_args: impl Iterator<Item = &'a String>,
    ) -> Self {
        let extension = chain_spec::jupiter::Extensions::try_get(&*para_config.chain_spec);
        let chain_id = extension.map(|e| e.relay_chain.clone());
        let base_path = para_config
            .base_path
            .as_ref()
            .map(|x| x.path().join("polkadot"));
        Self {
            base_path,
            chain_id,
            base: polkadot_cli::RunCmd::from_iter(relay_chain_args),
        }
    }
}
