use self::connection::BcConnection;
use crate::bc;
use log::*;
use std::convert::TryInto;
use std::net::ToSocketAddrs;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use Md5Trunc::*;

mod connection;
mod errors;
mod login;
mod logout;
mod media_packet;
mod ping;
mod stream;
mod time;
mod version;

pub use errors::Error;
pub use stream::{StreamFormat, StreamOutput, StreamOutputError};

type Result<T> = std::result::Result<T, Error>;

const RX_TIMEOUT: Duration = Duration::from_secs(5);

impl<'a> From<std::sync::mpsc::RecvTimeoutError> for Error {
    fn from(k: std::sync::mpsc::RecvTimeoutError) -> Self {
        match k {
            std::sync::mpsc::RecvTimeoutError::Timeout => Error::Timeout,
            std::sync::mpsc::RecvTimeoutError::Disconnected => Error::TimeoutDisconnected,
        }
    }
}

///
/// This is the primary struct of this library when interacting with the camera
///
pub struct BcCamera {
    channel_id: u8,
    connection: Option<BcConnection>,
    logged_in: bool,
    message_num: AtomicU16,
}

impl Drop for BcCamera {
    fn drop(&mut self) {
        self.disconnect();
    }
}

impl BcCamera {
    ///
    /// Create a new camera interface with this address and channel ID
    ///
    /// # Parameters
    ///
    /// * `host` - The address of the camera either ip address or hostname string is ok but
    ///             `SocketAddr` is fine too
    ///
    /// * `channel_id` - The channel ID this is usually zero unless using a NVR
    ///
    /// # Returns
    ///
    /// returns either an error of the camera
    ///
    pub fn new_with_addr<T: ToSocketAddrs>(host: T, channel_id: u8) -> Result<Self> {
        let addr_iter = match host.to_socket_addrs() {
            Ok(iter) => iter,
            Err(_) => return Err(Error::AddrResolutionError),
        };

        for addr in addr_iter {
            debug!("Trying {}", addr);
            let conn = match BcConnection::new(addr, RX_TIMEOUT) {
                Ok(conn) => conn,
                Err(err) => match err {
                    connection::Error::Communication(ref err) => {
                        debug!("Assuming timeout from {}", err);
                        continue;
                    }
                    err => return Err(err.into()),
                },
            };

            debug!("Success: {}", addr);
            return Ok(Self {
                connection: Some(conn),
                message_num: AtomicU16::new(0),
                channel_id,
                logged_in: false,
            });
        }

        Err(Error::Timeout)
    }

    /// This method will get a new message number and increment the message count atomically
    pub fn new_message_num(&self) -> u16 {
        self.message_num.fetch_add(1, Ordering::Relaxed)
    }

    /// This will drop the connection. It will try to send the logout request to the camera
    /// first
    pub fn disconnect(&mut self) {
        if let Err(err) = self.logout() {
            warn!("Could not log out, ignoring: {}", err);
        }
        self.connection = None;
    }
}

/// The Baichuan library has a very peculiar behavior where it always zeros the last byte.  I
/// believe this is because the MD5'ing of the user/password is a recent retrofit to the code and
/// the original code wanted to prevent a buffer overflow with strcpy.  The modern and legacy login
/// messages have a slightly different behavior; the legacy message has a 32-byte buffer and the
/// modern message uses XML.  The legacy code copies all 32 bytes with memcpy, and the XML value is
/// copied from a C-style string, so the appended null byte is dropped by the XML library - see the
/// test below.
/// Emulate this behavior by providing a configurable mangling of the last character.
#[derive(PartialEq, Eq)]
enum Md5Trunc {
    ZeroLast,
    Truncate,
}

fn md5_string(input: &str, trunc: Md5Trunc) -> String {
    let mut md5 = format!("{:X}\0", md5::compute(input));
    md5.replace_range(31.., if trunc == Truncate { "" } else { "\0" });
    md5
}

/// This is a convience function to make an AES key from the login password and the NONCE
/// negociated during login
pub fn make_aes_key(nonce: &str, passwd: &str) -> [u8; 16] {
    let key_phrase = format!("{}-{}", nonce, passwd);
    let key_phrase_hash = format!("{:X}\0", md5::compute(&key_phrase))
        .to_uppercase()
        .into_bytes();
    key_phrase_hash[0..16].try_into().unwrap()
}

#[test]
fn test_md5_string() {
    // Note that these literals are only 31 characters long - see explanation above.
    assert_eq!(
        md5_string("admin", Truncate),
        "21232F297A57A5A743894A0E4A801FC"
    );
    assert_eq!(
        md5_string("admin", ZeroLast),
        "21232F297A57A5A743894A0E4A801FC\0"
    );
}