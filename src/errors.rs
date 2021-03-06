
use cortical;
use ctrlc;
use sandbox;
use sc2;
use tantrum;

error_chain! {
    foreign_links {
        Ctrlc(ctrlc::Error);
    }
    links {
        Sc2(sc2::Error, sc2::ErrorKind);
        Cortical(cortical::Error, cortical::ErrorKind);
        Tantrum(tantrum::Error, tantrum::ErrorKind);
        Sandbox(sandbox::Error, sandbox::ErrorKind);
    }
}
