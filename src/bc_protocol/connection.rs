use crate::bc;
use crate::bc::model::*;
use err_derive::Error;
use log::*;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::net::{TcpStream, Shutdown, SocketAddr};
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};

/// A shareable connection to a camera.  Handles serialization of messages.  To send/receive, call
/// .subscribe() with a message ID.  You can use the BcSubscription to send or receive only
/// messages with that ID; each incoming message is routed to its appropriate subscriber.
///
/// There can be only one subscriber per kind of message at a time.
pub struct BcConnection {
    connection: Arc<Mutex<TcpStream>>,
    subscribers: Arc<Mutex<BTreeMap<u32, Sender<Bc>>>>,
    rx_thread: Option<JoinHandle<()>>,
}

pub struct BcSubscription<'a> {
    pub rx: Receiver<Bc>,
    msg_id: u32,
    conn: &'a BcConnection,
}

type Result<T> = std::result::Result<T, Error>;

***REMOVED***[derive(Debug, Error)]
pub enum Error {
    ***REMOVED***[error(display="Communication error")]
    CommunicationError(***REMOVED***[error(source)] std::io::Error),

    ***REMOVED***[error(display="Deserialization error")]
    DeserializationError(***REMOVED***[error(source)] bc::de::Error),

    ***REMOVED***[error(display="Serialization error")]
    SerializationError(***REMOVED***[error(source)] bc::ser::Error),

    ***REMOVED***[error(display="Simultaneous subscription")]
    SimultaneousSubscription { msg_id: u32 },
}

impl BcConnection {
    pub fn new(addr: SocketAddr) -> Result<BcConnection> {
        let tcp_conn = TcpStream::connect(addr)?;
        let subscribers: Arc<Mutex<BTreeMap<u32, Sender<Bc>>>> = Default::default();

        let mut subs = subscribers.clone();
        let conn = tcp_conn.try_clone()?;

        let rx_thread = std::thread::spawn(move || {
            let mut context = BcContext::new();
            while let Ok(_) = BcConnection::poll(&mut context, &conn, &mut subs) {}
        });

        Ok(BcConnection {
            connection: Arc::new(Mutex::new(tcp_conn)),
            subscribers,
            rx_thread: Some(rx_thread),
        })
    }

    pub fn subscribe(&self, msg_id: u32) -> Result<BcSubscription> {
        let (tx, rx) = channel();
        match self.subscribers.lock().unwrap().entry(msg_id) {
            Entry::Vacant(vac_entry) => vac_entry.insert(tx),
            Entry::Occupied(_) => return Err(Error::SimultaneousSubscription { msg_id }),
        };
        Ok(BcSubscription { rx, conn: self, msg_id })
    }

    fn poll(
        context: &mut BcContext,
        connection: &TcpStream,
        subscribers: &mut Arc<Mutex<BTreeMap<u32, Sender<Bc>>>>
    ) -> Result<()> {
        // Don't hold the lock during deserialization so we don't poison the subscribers mutex if
        // something goes wrong

        let response = Bc::deserialize(context, connection)
            .map_err(|err| {
                // If the connection hangs up, hang up on all subscribers
                subscribers.lock().unwrap().clear();
                err
            })?;
        let msg_id = response.meta.msg_id;

        let mut locked_subs = subscribers.lock().unwrap();
        match locked_subs.entry(msg_id) {
            Entry::Occupied(mut occ) => {
                if let Err(_) = occ.get_mut().send(response) {
                    // Exceedingly unlikely, unless you mishandle the subscription object
                    warn!("Subscriber to ID {} dropped their channel", msg_id);
                    occ.remove();
                }
            }
            Entry::Vacant(_) => {
                info!("Ignoring uninteresting message ID {}", msg_id);
                debug!("Contents: {:?}", response);
            }
        }

        Ok(())
    }
}

impl Drop for BcConnection {
    fn drop(&mut self) {
        debug!("Shutting down BcConnection...");
        let _ = self.connection.lock().unwrap().shutdown(Shutdown::Both);
        match self.rx_thread.take().expect("rx_thread join handle should always exist").join() {
            Ok(_) => {
                debug!("Shutdown finished OK");
            },
            Err(e) => {
                error!("Receiving thread panicked: {:?}", e);
            }
        }
    }
}

impl<'a> BcSubscription<'a> {
    pub fn send(&self, bc: Bc) -> Result<()> {
        assert!(bc.meta.msg_id == self.msg_id);

        bc.serialize(&*self.conn.connection.lock().unwrap())?;
        Ok(())
    }
}

/// Makes it difficult to avoid unsubscribing when you're finished
impl<'a> Drop for BcSubscription<'a> {
    fn drop(&mut self) {
        self.conn.subscribers.lock().unwrap().remove(&self.msg_id);
    }
}
