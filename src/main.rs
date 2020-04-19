***REMOVED***![allow(dead_code)] // while I'm still fleshing this out

use err_derive::Error;
use std::env::args;

mod bc;
mod bc_protocol;

use bc_protocol::BcCamera;

***REMOVED***[derive(Debug, Error)]
pub enum Error {
    ***REMOVED***[error(display="Communication error")]
    ProtocolError(***REMOVED***[error(source)] bc_protocol::Error),
}

fn main() -> Result<(), Error> {
    let mut camera = BcCamera::new_with_addr(args().nth(1).unwrap())?;
    camera.connect()?;
    camera.login("admin", Some("12345678"))?;
    Ok(())
}
