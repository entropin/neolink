use std::path::PathBuf;
use structopt::StructOpt;

/// A standards-compliant bridge to Reolink IP cameras
***REMOVED***[derive(StructOpt, Debug)]
***REMOVED***[structopt(name = "neolink")]
pub struct Opt {
    /// main configuration file
    ***REMOVED***[structopt(short, long, parse(from_os_str))]
    pub config: PathBuf,
}
