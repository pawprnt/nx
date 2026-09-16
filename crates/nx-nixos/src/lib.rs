pub mod args;
pub mod nixos;
pub mod generations;

use color_eyre::eyre::Result;

pub fn run(args: args::OsArgs) -> Result<()> {
    match args.action {
        args::OsAction::Rebuild(a) => nixos::rebuild(a),
        args::OsAction::Rollback(a) => nixos::rollback(a),
        args::OsAction::Info => nixos::info(),
        args::OsAction::Delete(a) => nixos::delete(a),
    }
}
