use std::path::PathBuf;

// basic CLI interface for LSIF dump
xflags::xflags! {
	src "./src/flags.rs"
    
	cmd lsif-p4-cmd {
        /// Requires the path to the system header files that are different for each P4 Switch arch
        optional -h, --header-files system_dest: PathBuf

        /// Optional path directory for workspace to be analyzers (default is current directory) 
        optional -w, --workspace input_dest: PathBuf

        /// Optional filename of LSIF file. File extension can't be changed (default name is P4Analysis)
        optional -f, --filename filename: String 

        /// Optional output directory of LSIF dump (default is current directory)
        optional -o, --output output_dest: PathBuf

        ///  Displays the version number.
		optional -v,--version
	}
}

// generated start
// The following code is generated by `xflags` macro.
// Run `env UPDATE_XFLAGS=1 cargo build` to regenerate.
#[derive(Debug)]
pub struct LsifP4Cmd {
    pub header_files: Option<PathBuf>,
    pub workspace: Option<PathBuf>,
    pub filename: Option<String>,
    pub output: Option<PathBuf>,
    pub version: bool,
}

impl LsifP4Cmd {
    #[allow(dead_code)]
    pub fn from_env_or_exit() -> Self {
        Self::from_env_or_exit_()
    }

    #[allow(dead_code)]
    pub fn from_env() -> xflags::Result<Self> {
        Self::from_env_()
    }

    #[allow(dead_code)]
    pub fn from_vec(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
        Self::from_vec_(args)
    }
}
// generated end
