
use cortical;
use sc2;
use tantrum;

error_chain! {
    errors {
        JoinError {
            description("an error occurred while joining a thread"),
            display("an error occurred while joining a thread")
        }
    }
    links {
        Sc2(sc2::Error, sc2::ErrorKind);
        Cortical(cortical::Error, cortical::ErrorKind);
        Tantrum(tantrum::Error, tantrum::ErrorKind);
    }
}
