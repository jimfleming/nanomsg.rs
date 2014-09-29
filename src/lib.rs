#![crate_type = "lib"]
#![license = "MIT/ASL2"]
#![feature(globs, unsafe_destructor, phase)]

#[phase(plugin, link)] extern crate log;

extern crate libc;

extern crate libnanomsg;

pub use result::{NanoResult, NanoError};

use libc::{c_int};
use std::kinds::marker::ContravariantLifetime;
use result::{SocketInitializationError, SocketBindError};

mod result;

/// Type-safe protocols that Nanomsg uses. Each socket
/// is bound to a single protocol that has specific behaviour
/// (such as only being able to receive messages and not send 'em).
#[deriving(Show, PartialEq)]
pub enum Protocol {
    Req,
    Rep,
    Push,
    Pull
}

/// A type-safe socket wrapper around nanomsg's own socket implementation. This
/// provides a safe interface for dealing with initializing the sockets, sending
/// and receiving messages.
pub struct Socket<'a> {
    addr: Option<&'a str>,
    socket: c_int,
    marker: ContravariantLifetime<'a>
}

impl<'a> Socket<'a> {

    /// Allocate and initialize a new Nanomsg socket which returns
    /// a new file descriptor behind the scene. The safe interface doesn't
    /// expose any of the underlying file descriptors and such.
    ///
    /// Usage:
    ///
    /// ```rust
    /// use nanomsg::{Socket, Pull};
    ///
    /// let mut socket = match Socket::new(Pull) {
    ///     Ok(socket) => socket,
    ///     Err(err) => fail!("{}", err)
    /// };
    /// ```
    pub fn new(protocol: Protocol) -> NanoResult<Socket<'a>> {

        let proto = match protocol {
            Req => libnanomsg::NN_REQ,
            Rep => libnanomsg::NN_REP,
            Push => libnanomsg::NN_PUSH,
            Pull => libnanomsg::NN_PULL
        };

        let socket = unsafe {
            libnanomsg::nn_socket(libnanomsg::AF_SP, proto)
        };

        if socket == -1 {
            return Err(NanoError::new("Failed to create a new nanomsg socket. Error: {}", SocketInitializationError));
        }

        debug!("Initialized a new raw socket");

        Ok(Socket {
            addr: None,
            socket: socket,
            marker: ContravariantLifetime::<'a>
        })
    }

    /// Creating a new socket through `Socket::new` does **not**
    /// bind that socket to a listening state. Instead, one has to be
    /// explicit in enabling the socket to listen onto a specific address.
    ///
    /// That's what the `bind` method does. Passing in a raw string like:
    /// "ipc:///tmp/pipeline.ipc" is supported.
    ///
    /// Note: This does **not** block the current task. That job
    /// is up to the user of the library by entering a loop.
    ///
    /// Usage:
    ///
    /// ```rust
    /// use nanomsg::{Socket, Pull};
    ///
    /// let mut socket = match Socket::new(Pull) {
    ///     Ok(socket) => socket,
    ///     Err(err) => fail!("{}", err)
    /// };
    ///
    /// // Bind the newly created socket to the following address:
    /// match socket.bind("ipc:///tmp/pipeline.ipc") {
    ///     Ok(_) => {},
    ///     Err(err) => fail!("Failed to bind socket: {}", err)
    /// }
    /// ```
    pub fn bind(&mut self, addr: &'a str) -> NanoResult<()> {
        let ret = unsafe { libnanomsg::nn_bind(self.socket, addr.as_ptr() as *const i8) };

        if ret == -1 {
            return Err(NanoError::new(format!("Failed to find the socket to the address: {}", addr), SocketBindError));
        }

        Ok(())
    }
}

#[unsafe_destructor]
impl<'a> Drop for Socket<'a> {
    fn drop(&mut self) {
        unsafe { libnanomsg::nn_shutdown(self.socket, 0); }
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_must_use)]
    extern crate debug;

    use super::*;

    #[test]
    fn initialize_socket() {
        let mut socket = match Socket::new(Pull) {
            Ok(socket) => socket,
            Err(err) => fail!("{}", err)
        };

        assert!(socket.socket >= 0);
    }

    #[test]
    fn bind_socket() {
        let mut socket = match Socket::new(Pull) {
            Ok(socket) => socket,
            Err(err) => fail!("{}", err)
        };

        match socket.bind("ipc:///tmp/pipeline.ipc") {
            Ok(_) => {},
            Err(err) => fail!("{}", err)
        }
    }
}
