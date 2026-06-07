use crate::{
    ce::{coredll::CoredllGuestMemory, kernel::CeKernel, memory::PROCESS_HEAP_HANDLE},
    error::Result,
};

use std::{
    collections::BTreeMap,
    io::{ErrorKind, Read, Write},
    net::{
        Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpListener, TcpStream, ToSocketAddrs,
        UdpSocket,
    },
    sync::{Mutex, OnceLock},
    time::Duration,
};

pub const INVALID_SOCKET: u32 = u32::MAX;
pub const SOCKET_ERROR: u32 = u32::MAX;
pub const WSAVERNOTSUPPORTED: u32 = 10092;

const WSADESCRIPTION_LEN: usize = 256;
const WSASYS_STATUS_LEN: usize = 128;
const AF_INET: u32 = 2;
const SOCK_STREAM: u32 = 1;
const SOCK_DGRAM: u32 = 2;
const FIONBIO: u32 = 0x8004_667e;
const SOCKET_HANDLE_BASE: u32 = 0x7100_0000;
const MAX_GUEST_IO: usize = 1024 * 1024;

const WSAEINTR: u32 = 10004;
const WSAEBADF: u32 = 10009;
const WSAEFAULT: u32 = 10014;
const WSAEINVAL: u32 = 10022;
const WSAEMFILE: u32 = 10024;
const WSAEWOULDBLOCK: u32 = 10035;
const WSAEADDRINUSE: u32 = 10048;
const WSAECONNRESET: u32 = 10054;
const WSAENOTCONN: u32 = 10057;
const WSAEOPNOTSUPP: u32 = 10045;
const WSAEAFNOSUPPORT: u32 = 10047;
const WSAHOST_NOT_FOUND: u32 = 11001;

pub fn dispatch_import<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    ordinal: Option<u32>,
    name: Option<&str>,
    memory: &mut M,
    args: &[u32],
) -> u32 {
    let normalized = name.map(normalize_symbol).unwrap_or_default();
    match (ordinal, normalized.as_str()) {
        (Some(3), _) | (_, "wsastartup") => {
            wsa_startup(memory, raw_import_arg(args, 0), raw_import_arg(args, 1))
        }
        (Some(1), _) | (_, "wsacleanup") => {
            set_wsa_error(kernel, thread_id, 0);
            0
        }
        (_, "wsagetlasterror") => kernel.threads.get_last_error(thread_id),
        (_, "wsasetlasterror") => {
            kernel
                .threads
                .set_last_error(thread_id, raw_import_arg(args, 0));
            0
        }
        (_, "socket" | "wsasocketw") => socket_raw(
            kernel,
            thread_id,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
        ),
        (_, "closesocket") => closesocket_raw(kernel, thread_id, raw_import_arg(args, 0)),
        (_, "connect" | "wsaconnect") => connect_raw(
            kernel,
            thread_id,
            memory,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
        ),
        (_, "bind") => bind_raw(
            kernel,
            thread_id,
            memory,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
        ),
        (_, "listen") => listen_raw(kernel, thread_id, raw_import_arg(args, 0)),
        (_, "accept" | "wsaaccept") => accept_raw(
            kernel,
            thread_id,
            memory,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
        ),
        (_, "send" | "wsasend") => send_raw(
            kernel,
            thread_id,
            memory,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
        ),
        (_, "recv" | "wsarecv") => recv_raw(
            kernel,
            thread_id,
            memory,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
        ),
        (_, "sendto" | "wsasendto") => sendto_raw(
            kernel,
            thread_id,
            memory,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
            raw_import_arg(args, 4),
            raw_import_arg(args, 5),
        ),
        (_, "recvfrom" | "wsarecvfrom") => recvfrom_raw(
            kernel,
            thread_id,
            memory,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
            raw_import_arg(args, 4),
            raw_import_arg(args, 5),
        ),
        (_, "select") => select_raw(
            kernel,
            thread_id,
            memory,
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
            raw_import_arg(args, 3),
        ),
        (_, "__wsafdisset") => {
            fd_is_set_raw(memory, raw_import_arg(args, 0), raw_import_arg(args, 1))
        }
        (_, "ioctlsocket") => ioctlsocket_raw(
            kernel,
            thread_id,
            memory,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
        ),
        (_, "setsockopt") | (_, "getsockopt") => {
            set_wsa_error(kernel, thread_id, 0);
            0
        }
        (_, "shutdown") => shutdown_raw(
            kernel,
            thread_id,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
        ),
        (_, "htonl" | "ntohl") => raw_import_arg(args, 0).swap_bytes(),
        (_, "htons" | "ntohs") => u32::from((raw_import_arg(args, 0) as u16).swap_bytes()),
        (_, "inet_addr") => inet_addr_raw(kernel, thread_id, memory, raw_import_arg(args, 0)),
        (_, "inet_ntoa") => inet_ntoa_raw(kernel, thread_id, memory, raw_import_arg(args, 0)),
        (_, "gethostname") => gethostname_raw(
            kernel,
            thread_id,
            memory,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
        ),
        (_, "gethostbyname") => {
            gethostbyname_raw(kernel, thread_id, memory, raw_import_arg(args, 0))
        }
        _ => {
            set_wsa_error(kernel, thread_id, WSAEOPNOTSUPP);
            SOCKET_ERROR
        }
    }
}

pub fn wsa_startup<M: CoredllGuestMemory>(memory: &mut M, requested: u32, data_ptr: u32) -> u32 {
    if data_ptr == 0 {
        return WSAVERNOTSUPPORTED;
    }
    let major = ((requested >> 8) & 0xff).clamp(1, 2) as u16;
    let minor = (requested & 0xff).min(2) as u16;
    let version = (major << 8) | minor;
    if memory.write_u16(data_ptr, version).is_err()
        || memory.write_u16(data_ptr + 2, 0x0202).is_err()
        || write_guest_bytes(
            memory,
            data_ptr + 4,
            b"FakeCE Winsock\0",
            WSADESCRIPTION_LEN + 1,
        )
        .is_err()
        || write_guest_bytes(
            memory,
            data_ptr + 4 + 257,
            b"Running\0",
            WSASYS_STATUS_LEN + 1,
        )
        .is_err()
        || memory.write_u16(data_ptr + 4 + 257 + 129, 0).is_err()
        || memory.write_u16(data_ptr + 4 + 257 + 129 + 2, 0).is_err()
        || memory.write_u32(data_ptr + 4 + 257 + 129 + 4, 0).is_err()
    {
        return WSAVERNOTSUPPORTED;
    }
    0
}

enum HostSocket {
    Pending {
        family: u32,
        socket_type: u32,
        protocol: u32,
        nonblocking: bool,
    },
    TcpStream(TcpStream),
    TcpListener(TcpListener),
    Udp(UdpSocket),
}

#[derive(Default)]
struct WinsockState {
    next_socket: u32,
    sockets: BTreeMap<u32, HostSocket>,
}

impl WinsockState {
    fn allocate(&mut self, socket: HostSocket) -> Option<u32> {
        if self.next_socket < SOCKET_HANDLE_BASE {
            self.next_socket = SOCKET_HANDLE_BASE;
        }
        for _ in 0..0x000f_ffff {
            let handle = self.next_socket;
            self.next_socket = self.next_socket.wrapping_add(1);
            if handle != INVALID_SOCKET && !self.sockets.contains_key(&handle) {
                self.sockets.insert(handle, socket);
                return Some(handle);
            }
        }
        None
    }
}

static WINSOCK_STATE: OnceLock<Mutex<WinsockState>> = OnceLock::new();

fn state() -> &'static Mutex<WinsockState> {
    WINSOCK_STATE.get_or_init(|| Mutex::new(WinsockState::default()))
}

fn socket_raw(
    kernel: &mut CeKernel,
    thread_id: u32,
    family: u32,
    socket_type: u32,
    protocol: u32,
) -> u32 {
    if family != AF_INET {
        set_wsa_error(kernel, thread_id, WSAEAFNOSUPPORT);
        return INVALID_SOCKET;
    }
    if socket_type != SOCK_STREAM && socket_type != SOCK_DGRAM {
        set_wsa_error(kernel, thread_id, WSAEOPNOTSUPP);
        return INVALID_SOCKET;
    }
    let socket = HostSocket::Pending {
        family,
        socket_type,
        protocol,
        nonblocking: false,
    };
    match state()
        .lock()
        .ok()
        .and_then(|mut state| state.allocate(socket))
    {
        Some(handle) => {
            set_wsa_error(kernel, thread_id, 0);
            handle
        }
        None => {
            set_wsa_error(kernel, thread_id, WSAEMFILE);
            INVALID_SOCKET
        }
    }
}

fn closesocket_raw(kernel: &mut CeKernel, thread_id: u32, socket: u32) -> u32 {
    let removed = state()
        .lock()
        .ok()
        .and_then(|mut state| state.sockets.remove(&socket));
    if removed.is_some() {
        set_wsa_error(kernel, thread_id, 0);
        0
    } else {
        set_wsa_error(kernel, thread_id, WSAEBADF);
        SOCKET_ERROR
    }
}

fn connect_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    socket: u32,
    name_ptr: u32,
    name_len: u32,
) -> u32 {
    let addr = match read_sockaddr_in(memory, name_ptr, name_len) {
        Ok(addr) => addr,
        Err(error) => {
            set_wsa_error(kernel, thread_id, error);
            return SOCKET_ERROR;
        }
    };
    let mut state = match state().lock() {
        Ok(state) => state,
        Err(_) => {
            set_wsa_error(kernel, thread_id, WSAEINTR);
            return SOCKET_ERROR;
        }
    };
    let Some(entry) = state.sockets.get_mut(&socket) else {
        set_wsa_error(kernel, thread_id, WSAEBADF);
        return SOCKET_ERROR;
    };
    let result = match entry {
        HostSocket::Pending {
            family,
            socket_type,
            protocol,
            nonblocking,
        } if *family == AF_INET && *socket_type == SOCK_STREAM => {
            let timeout = if *nonblocking {
                Duration::from_millis(1)
            } else {
                Duration::from_secs(3)
            };
            match TcpStream::connect_timeout(&addr, timeout) {
                Ok(stream) => {
                    configure_stream(&stream, *nonblocking);
                    *entry = HostSocket::TcpStream(stream);
                    Ok(())
                }
                Err(error) => Err(io_to_wsa_error(&error)),
            }
        }
        HostSocket::Pending {
            family,
            socket_type,
            protocol: _,
            nonblocking,
        } if *family == AF_INET && *socket_type == SOCK_DGRAM => {
            match UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)) {
                Ok(udp) => {
                    let _ = udp.set_nonblocking(*nonblocking);
                    match udp.connect(addr) {
                        Ok(()) => {
                            *entry = HostSocket::Udp(udp);
                            Ok(())
                        }
                        Err(error) => Err(io_to_wsa_error(&error)),
                    }
                }
                Err(error) => Err(io_to_wsa_error(&error)),
            }
        }
        HostSocket::TcpStream(_) | HostSocket::Udp(_) => Ok(()),
        _ => Err(WSAEOPNOTSUPP),
    };
    finish_socket_result(kernel, thread_id, result)
}

fn bind_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    socket: u32,
    name_ptr: u32,
    name_len: u32,
) -> u32 {
    let addr = match read_sockaddr_in(memory, name_ptr, name_len) {
        Ok(addr) => addr,
        Err(error) => {
            set_wsa_error(kernel, thread_id, error);
            return SOCKET_ERROR;
        }
    };
    let mut state = match state().lock() {
        Ok(state) => state,
        Err(_) => {
            set_wsa_error(kernel, thread_id, WSAEINTR);
            return SOCKET_ERROR;
        }
    };
    let Some(entry) = state.sockets.get_mut(&socket) else {
        set_wsa_error(kernel, thread_id, WSAEBADF);
        return SOCKET_ERROR;
    };
    let result = match entry {
        HostSocket::Pending {
            family,
            socket_type,
            nonblocking,
            ..
        } if *family == AF_INET && *socket_type == SOCK_STREAM => match TcpListener::bind(addr) {
            Ok(listener) => {
                let _ = listener.set_nonblocking(true);
                *entry = HostSocket::TcpListener(listener);
                Ok(())
            }
            Err(error) => Err(io_to_wsa_error(&error)),
        },
        HostSocket::Pending {
            family,
            socket_type,
            nonblocking,
            ..
        } if *family == AF_INET && *socket_type == SOCK_DGRAM => match UdpSocket::bind(addr) {
            Ok(udp) => {
                let _ = udp.set_nonblocking(*nonblocking);
                *entry = HostSocket::Udp(udp);
                Ok(())
            }
            Err(error) => Err(io_to_wsa_error(&error)),
        },
        _ => Err(WSAEINVAL),
    };
    finish_socket_result(kernel, thread_id, result)
}

fn listen_raw(kernel: &mut CeKernel, thread_id: u32, socket: u32) -> u32 {
    let ok = state()
        .lock()
        .ok()
        .and_then(|state| {
            state
                .sockets
                .get(&socket)
                .map(|socket| matches!(socket, HostSocket::TcpListener(_)))
        })
        .unwrap_or(false);
    if ok {
        set_wsa_error(kernel, thread_id, 0);
        0
    } else {
        set_wsa_error(kernel, thread_id, WSAEINVAL);
        SOCKET_ERROR
    }
}

fn accept_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    socket: u32,
    addr_ptr: u32,
    addr_len_ptr: u32,
) -> u32 {
    let accepted = {
        let mut state = match state().lock() {
            Ok(state) => state,
            Err(_) => {
                set_wsa_error(kernel, thread_id, WSAEINTR);
                return INVALID_SOCKET;
            }
        };
        let Some(HostSocket::TcpListener(listener)) = state.sockets.get(&socket) else {
            set_wsa_error(kernel, thread_id, WSAEBADF);
            return INVALID_SOCKET;
        };
        match listener.accept() {
            Ok((stream, remote)) => {
                configure_stream(&stream, true);
                let handle = match state.allocate(HostSocket::TcpStream(stream)) {
                    Some(handle) => handle,
                    None => {
                        set_wsa_error(kernel, thread_id, WSAEMFILE);
                        return INVALID_SOCKET;
                    }
                };
                (handle, remote)
            }
            Err(error) => {
                set_wsa_error(kernel, thread_id, io_to_wsa_error(&error));
                return INVALID_SOCKET;
            }
        }
    };
    if addr_ptr != 0 && addr_len_ptr != 0 {
        let _ = write_sockaddr_in(memory, addr_ptr, addr_len_ptr, accepted.1);
    }
    set_wsa_error(kernel, thread_id, 0);
    accepted.0
}

fn send_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    socket: u32,
    buf_ptr: u32,
    len: u32,
) -> u32 {
    let len = len.min(MAX_GUEST_IO as u32) as usize;
    let mut bytes = vec![0; len];
    if memory.read_bytes(buf_ptr, &mut bytes).is_err() {
        set_wsa_error(kernel, thread_id, WSAEFAULT);
        return SOCKET_ERROR;
    }
    let mut state = match state().lock() {
        Ok(state) => state,
        Err(_) => {
            set_wsa_error(kernel, thread_id, WSAEINTR);
            return SOCKET_ERROR;
        }
    };
    let result = match state.sockets.get_mut(&socket) {
        Some(HostSocket::TcpStream(stream)) => stream.write(&bytes),
        Some(HostSocket::Udp(udp)) => udp.send(&bytes),
        Some(_) => {
            set_wsa_error(kernel, thread_id, WSAENOTCONN);
            return SOCKET_ERROR;
        }
        None => {
            set_wsa_error(kernel, thread_id, WSAEBADF);
            return SOCKET_ERROR;
        }
    };
    finish_socket_count(kernel, thread_id, result)
}

fn recv_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    socket: u32,
    buf_ptr: u32,
    len: u32,
) -> u32 {
    let len = len.min(MAX_GUEST_IO as u32) as usize;
    let mut bytes = vec![0; len];
    let mut state = match state().lock() {
        Ok(state) => state,
        Err(_) => {
            set_wsa_error(kernel, thread_id, WSAEINTR);
            return SOCKET_ERROR;
        }
    };
    let result = match state.sockets.get_mut(&socket) {
        Some(HostSocket::TcpStream(stream)) => stream.read(&mut bytes),
        Some(HostSocket::Udp(udp)) => udp.recv(&mut bytes),
        Some(_) => {
            set_wsa_error(kernel, thread_id, WSAENOTCONN);
            return SOCKET_ERROR;
        }
        None => {
            set_wsa_error(kernel, thread_id, WSAEBADF);
            return SOCKET_ERROR;
        }
    };
    match result {
        Ok(count) => {
            if memory.write_bytes(buf_ptr, &bytes[..count]).is_err() {
                set_wsa_error(kernel, thread_id, WSAEFAULT);
                SOCKET_ERROR
            } else {
                set_wsa_error(kernel, thread_id, 0);
                count as u32
            }
        }
        Err(error) => {
            set_wsa_error(kernel, thread_id, io_to_wsa_error(&error));
            SOCKET_ERROR
        }
    }
}

fn sendto_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    socket: u32,
    buf_ptr: u32,
    len: u32,
    to_ptr: u32,
    to_len: u32,
) -> u32 {
    if to_ptr == 0 {
        return send_raw(kernel, thread_id, memory, socket, buf_ptr, len);
    }
    let addr = match read_sockaddr_in(memory, to_ptr, to_len) {
        Ok(addr) => addr,
        Err(error) => {
            set_wsa_error(kernel, thread_id, error);
            return SOCKET_ERROR;
        }
    };
    let len = len.min(MAX_GUEST_IO as u32) as usize;
    let mut bytes = vec![0; len];
    if memory.read_bytes(buf_ptr, &mut bytes).is_err() {
        set_wsa_error(kernel, thread_id, WSAEFAULT);
        return SOCKET_ERROR;
    }
    let mut state = match state().lock() {
        Ok(state) => state,
        Err(_) => {
            set_wsa_error(kernel, thread_id, WSAEINTR);
            return SOCKET_ERROR;
        }
    };
    let result = match state.sockets.get_mut(&socket) {
        Some(HostSocket::Udp(udp)) => udp.send_to(&bytes, addr),
        Some(_) => {
            set_wsa_error(kernel, thread_id, WSAEOPNOTSUPP);
            return SOCKET_ERROR;
        }
        None => {
            set_wsa_error(kernel, thread_id, WSAEBADF);
            return SOCKET_ERROR;
        }
    };
    finish_socket_count(kernel, thread_id, result)
}

fn recvfrom_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    socket: u32,
    buf_ptr: u32,
    len: u32,
    from_ptr: u32,
    from_len_ptr: u32,
) -> u32 {
    let len = len.min(MAX_GUEST_IO as u32) as usize;
    let mut bytes = vec![0; len];
    let mut state = match state().lock() {
        Ok(state) => state,
        Err(_) => {
            set_wsa_error(kernel, thread_id, WSAEINTR);
            return SOCKET_ERROR;
        }
    };
    let result = match state.sockets.get_mut(&socket) {
        Some(HostSocket::Udp(udp)) => udp.recv_from(&mut bytes),
        Some(_) => {
            set_wsa_error(kernel, thread_id, WSAEOPNOTSUPP);
            return SOCKET_ERROR;
        }
        None => {
            set_wsa_error(kernel, thread_id, WSAEBADF);
            return SOCKET_ERROR;
        }
    };
    match result {
        Ok((count, from)) => {
            if memory.write_bytes(buf_ptr, &bytes[..count]).is_err() {
                set_wsa_error(kernel, thread_id, WSAEFAULT);
                return SOCKET_ERROR;
            }
            if from_ptr != 0 && from_len_ptr != 0 {
                let _ = write_sockaddr_in(memory, from_ptr, from_len_ptr, from);
            }
            set_wsa_error(kernel, thread_id, 0);
            count as u32
        }
        Err(error) => {
            set_wsa_error(kernel, thread_id, io_to_wsa_error(&error));
            SOCKET_ERROR
        }
    }
}

fn select_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    readfds_ptr: u32,
    writefds_ptr: u32,
    exceptfds_ptr: u32,
) -> u32 {
    let mut ready = 0;
    ready += filter_fd_set(memory, readfds_ptr, |socket| socket_read_ready(socket)).unwrap_or(0);
    ready += filter_fd_set(memory, writefds_ptr, |socket| socket_write_ready(socket)).unwrap_or(0);
    if exceptfds_ptr != 0 {
        let _ = memory.write_u32(exceptfds_ptr, 0);
    }
    set_wsa_error(kernel, thread_id, 0);
    ready
}

fn fd_is_set_raw<M: CoredllGuestMemory>(memory: &mut M, socket: u32, fdset_ptr: u32) -> u32 {
    if fdset_ptr == 0 {
        return 0;
    }
    let Ok(count) = memory.read_u32(fdset_ptr) else {
        return 0;
    };
    for index in 0..count.min(64) {
        if memory.read_u32(fdset_ptr + 4 + index * 4).ok() == Some(socket) {
            return 1;
        }
    }
    0
}

fn ioctlsocket_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    socket: u32,
    cmd: u32,
    argp: u32,
) -> u32 {
    if cmd != FIONBIO || argp == 0 {
        set_wsa_error(kernel, thread_id, WSAEINVAL);
        return SOCKET_ERROR;
    }
    let nonblocking = memory.read_u32(argp).unwrap_or(0) != 0;
    let mut state = match state().lock() {
        Ok(state) => state,
        Err(_) => {
            set_wsa_error(kernel, thread_id, WSAEINTR);
            return SOCKET_ERROR;
        }
    };
    let Some(entry) = state.sockets.get_mut(&socket) else {
        set_wsa_error(kernel, thread_id, WSAEBADF);
        return SOCKET_ERROR;
    };
    match entry {
        HostSocket::Pending {
            nonblocking: mode, ..
        } => *mode = nonblocking,
        HostSocket::TcpStream(stream) => {
            let _ = stream.set_nonblocking(nonblocking);
        }
        HostSocket::TcpListener(listener) => {
            let _ = listener.set_nonblocking(nonblocking);
        }
        HostSocket::Udp(udp) => {
            let _ = udp.set_nonblocking(nonblocking);
        }
    }
    set_wsa_error(kernel, thread_id, 0);
    0
}

fn shutdown_raw(kernel: &mut CeKernel, thread_id: u32, socket: u32, how: u32) -> u32 {
    let shutdown = match how {
        0 => Shutdown::Read,
        1 => Shutdown::Write,
        _ => Shutdown::Both,
    };
    let mut state = match state().lock() {
        Ok(state) => state,
        Err(_) => {
            set_wsa_error(kernel, thread_id, WSAEINTR);
            return SOCKET_ERROR;
        }
    };
    let result = match state.sockets.get_mut(&socket) {
        Some(HostSocket::TcpStream(stream)) => stream.shutdown(shutdown).map(|_| ()),
        Some(_) => Err(std::io::Error::from(ErrorKind::Unsupported)),
        None => {
            set_wsa_error(kernel, thread_id, WSAEBADF);
            return SOCKET_ERROR;
        }
    };
    finish_socket_result(
        kernel,
        thread_id,
        result.map_err(|error| io_to_wsa_error(&error)),
    )
}

fn inet_addr_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    name_ptr: u32,
) -> u32 {
    let Some(name) = read_guest_c_string(memory, name_ptr, 256) else {
        set_wsa_error(kernel, thread_id, WSAEFAULT);
        return u32::MAX;
    };
    match name.parse::<Ipv4Addr>() {
        Ok(addr) => {
            set_wsa_error(kernel, thread_id, 0);
            u32::from_le_bytes(addr.octets())
        }
        Err(_) => {
            set_wsa_error(kernel, thread_id, WSAEINVAL);
            u32::MAX
        }
    }
}

fn inet_ntoa_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    addr: u32,
) -> u32 {
    let text = Ipv4Addr::from(addr.to_le_bytes()).to_string();
    write_heap_c_string(kernel, memory, thread_id, &text).unwrap_or(0)
}

fn gethostname_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    name_ptr: u32,
    name_len: u32,
) -> u32 {
    if name_ptr == 0 || name_len == 0 {
        set_wsa_error(kernel, thread_id, WSAEFAULT);
        return SOCKET_ERROR;
    }
    let name = b"fakece\0";
    let count = name.len().min(name_len as usize);
    if memory.write_bytes(name_ptr, &name[..count]).is_err() {
        set_wsa_error(kernel, thread_id, WSAEFAULT);
        return SOCKET_ERROR;
    }
    if count == name_len as usize {
        let _ = memory.write_u8(name_ptr + name_len - 1, 0);
    }
    set_wsa_error(kernel, thread_id, 0);
    0
}

fn gethostbyname_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    name_ptr: u32,
) -> u32 {
    let Some(name) = read_guest_c_string(memory, name_ptr, 256) else {
        set_wsa_error(kernel, thread_id, WSAEFAULT);
        return 0;
    };
    let addr = if let Ok(ip) = name.parse::<Ipv4Addr>() {
        ip
    } else {
        match (name.as_str(), 0)
            .to_socket_addrs()
            .ok()
            .and_then(|mut addrs| {
                addrs.find_map(|addr| match addr {
                    SocketAddr::V4(addr) => Some(*addr.ip()),
                    SocketAddr::V6(_) => None,
                })
            }) {
            Some(addr) => addr,
            None => {
                set_wsa_error(kernel, thread_id, WSAHOST_NOT_FOUND);
                return 0;
            }
        }
    };
    match write_hostent(kernel, memory, &name, addr) {
        Some(ptr) => {
            set_wsa_error(kernel, thread_id, 0);
            ptr
        }
        None => {
            set_wsa_error(kernel, thread_id, WSAEFAULT);
            0
        }
    }
}

fn filter_fd_set<M, F>(memory: &mut M, fdset_ptr: u32, ready_fn: F) -> Result<u32>
where
    M: CoredllGuestMemory,
    F: Fn(&HostSocket) -> bool,
{
    if fdset_ptr == 0 {
        return Ok(0);
    }
    let count = memory.read_u32(fdset_ptr)?.min(64);
    let mut ready_sockets = Vec::new();
    let state = state().lock().ok();
    for index in 0..count {
        let socket = memory.read_u32(fdset_ptr + 4 + index * 4)?;
        if state
            .as_ref()
            .and_then(|state| state.sockets.get(&socket))
            .is_some_and(&ready_fn)
        {
            ready_sockets.push(socket);
        }
    }
    memory.write_u32(fdset_ptr, ready_sockets.len() as u32)?;
    for (index, socket) in ready_sockets.iter().copied().enumerate() {
        memory.write_u32(fdset_ptr + 4 + index as u32 * 4, socket)?;
    }
    Ok(ready_sockets.len() as u32)
}

fn socket_read_ready(socket: &HostSocket) -> bool {
    match socket {
        HostSocket::TcpStream(stream) => {
            let mut byte = [0; 1];
            matches!(stream.peek(&mut byte), Ok(count) if count > 0)
        }
        HostSocket::Udp(_) | HostSocket::TcpListener(_) => true,
        HostSocket::Pending { .. } => false,
    }
}

fn socket_write_ready(socket: &HostSocket) -> bool {
    matches!(socket, HostSocket::TcpStream(_) | HostSocket::Udp(_))
}

fn read_sockaddr_in<M: CoredllGuestMemory>(
    memory: &mut M,
    ptr: u32,
    len: u32,
) -> std::result::Result<SocketAddr, u32> {
    if ptr == 0 || len < 16 {
        return Err(WSAEFAULT);
    }
    if memory.read_u16(ptr).map_err(|_| WSAEFAULT)? as u32 != AF_INET {
        return Err(WSAEAFNOSUPPORT);
    }
    let port_hi = memory.read_u8(ptr + 2).map_err(|_| WSAEFAULT)?;
    let port_lo = memory.read_u8(ptr + 3).map_err(|_| WSAEFAULT)?;
    let port = u16::from_be_bytes([port_hi, port_lo]);
    let ip = Ipv4Addr::new(
        memory.read_u8(ptr + 4).map_err(|_| WSAEFAULT)?,
        memory.read_u8(ptr + 5).map_err(|_| WSAEFAULT)?,
        memory.read_u8(ptr + 6).map_err(|_| WSAEFAULT)?,
        memory.read_u8(ptr + 7).map_err(|_| WSAEFAULT)?,
    );
    Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)))
}

fn write_sockaddr_in<M: CoredllGuestMemory>(
    memory: &mut M,
    ptr: u32,
    len_ptr: u32,
    addr: SocketAddr,
) -> Result<()> {
    let len = memory.read_u32(len_ptr)?;
    if len < 16 {
        return Ok(());
    }
    let SocketAddr::V4(addr) = addr else {
        return Ok(());
    };
    memory.write_u16(ptr, AF_INET as u16)?;
    let port = addr.port().to_be_bytes();
    memory.write_u8(ptr + 2, port[0])?;
    memory.write_u8(ptr + 3, port[1])?;
    memory.write_bytes(ptr + 4, &addr.ip().octets())?;
    memory.fill_bytes(ptr + 8, 0, 8)?;
    memory.write_u32(len_ptr, 16)?;
    Ok(())
}

fn write_hostent<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    name: &str,
    addr: Ipv4Addr,
) -> Option<u32> {
    let name_ptr = alloc_guest(kernel, name.len() as u32 + 1)?;
    memory.write_bytes(name_ptr, name.as_bytes()).ok()?;
    memory.write_u8(name_ptr + name.len() as u32, 0).ok()?;
    let aliases_ptr = alloc_guest(kernel, 4)?;
    memory.write_u32(aliases_ptr, 0).ok()?;
    let addr_ptr = alloc_guest(kernel, 4)?;
    memory.write_bytes(addr_ptr, &addr.octets()).ok()?;
    let addr_list_ptr = alloc_guest(kernel, 8)?;
    memory.write_u32(addr_list_ptr, addr_ptr).ok()?;
    memory.write_u32(addr_list_ptr + 4, 0).ok()?;
    let hostent_ptr = alloc_guest(kernel, 16)?;
    memory.write_u32(hostent_ptr, name_ptr).ok()?;
    memory.write_u32(hostent_ptr + 4, aliases_ptr).ok()?;
    memory.write_u16(hostent_ptr + 8, AF_INET as u16).ok()?;
    memory.write_u16(hostent_ptr + 10, 4).ok()?;
    memory.write_u32(hostent_ptr + 12, addr_list_ptr).ok()?;
    Some(hostent_ptr)
}

fn write_heap_c_string<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    memory: &mut M,
    thread_id: u32,
    text: &str,
) -> Option<u32> {
    let ptr = alloc_guest(kernel, text.len() as u32 + 1)?;
    memory.write_bytes(ptr, text.as_bytes()).ok()?;
    memory.write_u8(ptr + text.len() as u32, 0).ok()?;
    set_wsa_error(kernel, thread_id, 0);
    Some(ptr)
}

fn alloc_guest(kernel: &mut CeKernel, bytes: u32) -> Option<u32> {
    kernel.memory.heap_alloc(PROCESS_HEAP_HANDLE, 0, bytes)
}

fn read_guest_c_string<M: CoredllGuestMemory>(
    memory: &mut M,
    ptr: u32,
    max_len: usize,
) -> Option<String> {
    if ptr == 0 {
        return None;
    }
    let mut bytes = Vec::new();
    for offset in 0..max_len {
        let byte = memory.read_u8(ptr + offset as u32).ok()?;
        if byte == 0 {
            return String::from_utf8(bytes).ok();
        }
        bytes.push(byte);
    }
    None
}

fn configure_stream(stream: &TcpStream, nonblocking: bool) {
    let _ = stream.set_nonblocking(nonblocking);
    if !nonblocking {
        let _ = stream.set_read_timeout(Some(Duration::from_millis(1)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
    }
}

fn finish_socket_result(
    kernel: &mut CeKernel,
    thread_id: u32,
    result: std::result::Result<(), u32>,
) -> u32 {
    match result {
        Ok(()) => {
            set_wsa_error(kernel, thread_id, 0);
            0
        }
        Err(error) => {
            set_wsa_error(kernel, thread_id, error);
            SOCKET_ERROR
        }
    }
}

fn finish_socket_count(
    kernel: &mut CeKernel,
    thread_id: u32,
    result: std::io::Result<usize>,
) -> u32 {
    match result {
        Ok(count) => {
            set_wsa_error(kernel, thread_id, 0);
            count as u32
        }
        Err(error) => {
            set_wsa_error(kernel, thread_id, io_to_wsa_error(&error));
            SOCKET_ERROR
        }
    }
}

fn set_wsa_error(kernel: &mut CeKernel, thread_id: u32, error: u32) {
    kernel.threads.set_last_error(thread_id, error);
}

fn io_to_wsa_error(error: &std::io::Error) -> u32 {
    match error.kind() {
        ErrorKind::WouldBlock | ErrorKind::TimedOut => WSAEWOULDBLOCK,
        ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted | ErrorKind::BrokenPipe => {
            WSAECONNRESET
        }
        ErrorKind::AddrInUse => WSAEADDRINUSE,
        ErrorKind::InvalidInput | ErrorKind::InvalidData => WSAEINVAL,
        ErrorKind::NotConnected => WSAENOTCONN,
        ErrorKind::Unsupported => WSAEOPNOTSUPP,
        _ => WSAEINVAL,
    }
}

fn raw_import_arg(args: &[u32], index: usize) -> u32 {
    args.get(index).copied().unwrap_or(0)
}

fn normalize_symbol(name: &str) -> String {
    name.trim_start_matches('_')
        .split('@')
        .next()
        .unwrap_or(name)
        .to_ascii_lowercase()
}

fn write_guest_bytes<M: CoredllGuestMemory>(
    memory: &mut M,
    addr: u32,
    bytes: &[u8],
    capacity: usize,
) -> Result<()> {
    for index in 0..capacity {
        let value = bytes.get(index).copied().unwrap_or(0);
        memory.write_u8(addr + index as u32, value)?;
    }
    Ok(())
}

#[cfg(test)]
fn reset_for_tests() {
    *state().lock().unwrap() = WinsockState::default();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Result, config::RuntimeConfig};
    use std::{collections::BTreeMap, io::Read, net::TcpListener, thread};

    #[derive(Default)]
    struct TestMemory {
        bytes: BTreeMap<u32, u8>,
    }

    impl TestMemory {
        fn read_c_string(&self, addr: u32) -> String {
            let mut out = Vec::new();
            for offset in 0..512 {
                let value = self.bytes.get(&(addr + offset)).copied().unwrap_or(0);
                if value == 0 {
                    break;
                }
                out.push(value);
            }
            String::from_utf8(out).unwrap()
        }

        fn write_bytes_at(&mut self, addr: u32, bytes: &[u8]) {
            for (offset, byte) in bytes.iter().copied().enumerate() {
                self.bytes.insert(addr + offset as u32, byte);
            }
        }

        fn write_sockaddr_v4(&mut self, addr: u32, ip: Ipv4Addr, port: u16) {
            self.write_u16(addr, AF_INET as u16).unwrap();
            let port = port.to_be_bytes();
            self.write_u8(addr + 2, port[0]).unwrap();
            self.write_u8(addr + 3, port[1]).unwrap();
            self.write_bytes(addr + 4, &ip.octets()).unwrap();
            self.fill_bytes(addr + 8, 0, 8).unwrap();
        }
    }

    impl CoredllGuestMemory for TestMemory {
        fn read_u8(&self, addr: u32) -> Result<u8> {
            Ok(self.bytes.get(&addr).copied().unwrap_or(0))
        }

        fn write_u8(&mut self, addr: u32, value: u8) -> Result<()> {
            self.bytes.insert(addr, value);
            Ok(())
        }

        fn read_u32(&self, addr: u32) -> Result<u32> {
            let b0 = self.read_u8(addr)?;
            let b1 = self.read_u8(addr + 1)?;
            let b2 = self.read_u8(addr + 2)?;
            let b3 = self.read_u8(addr + 3)?;
            Ok(u32::from_le_bytes([b0, b1, b2, b3]))
        }

        fn write_u32(&mut self, addr: u32, value: u32) -> Result<()> {
            for (offset, byte) in value.to_le_bytes().into_iter().enumerate() {
                self.write_u8(addr + offset as u32, byte)?;
            }
            Ok(())
        }

        fn read_u16(&self, addr: u32) -> Result<u16> {
            let b0 = self.read_u8(addr)?;
            let b1 = self.read_u8(addr + 1)?;
            Ok(u16::from_le_bytes([b0, b1]))
        }

        fn write_u16(&mut self, addr: u32, value: u16) -> Result<()> {
            for (offset, byte) in value.to_le_bytes().into_iter().enumerate() {
                self.write_u8(addr + offset as u32, byte)?;
            }
            Ok(())
        }
    }

    fn test_kernel() -> CeKernel {
        CeKernel::boot(RuntimeConfig::load("regs.json", "serial_devices.json").unwrap())
    }

    #[test]
    fn wsa_startup_writes_ce_shaped_wsadata() {
        reset_for_tests();
        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let data = 0x3000_0000;

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("WSAStartup"),
                &mut memory,
                &[0x0202, data]
            ),
            0
        );

        assert_eq!(memory.read_u16(data).unwrap(), 0x0202);
        assert_eq!(memory.read_u16(data + 2).unwrap(), 0x0202);
        assert_eq!(memory.read_c_string(data + 4), "FakeCE Winsock");
        assert_eq!(memory.read_c_string(data + 4 + 257), "Running");
        assert_eq!(memory.read_u16(data + 4 + 257 + 129).unwrap(), 0);
        assert_eq!(memory.read_u16(data + 4 + 257 + 129 + 2).unwrap(), 0);
        assert_eq!(memory.read_u32(data + 4 + 257 + 129 + 4).unwrap(), 0);
    }

    #[test]
    fn wsa_startup_rejects_null_wsadata_pointer() {
        reset_for_tests();
        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();

        assert_eq!(
            dispatch_import(&mut kernel, 1, Some(3), None, &mut memory, &[0x0202, 0]),
            WSAVERNOTSUPPORTED
        );
    }

    #[test]
    fn host_tcp_socket_connect_send_and_recv_use_loopback() {
        reset_for_tests();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut input = [0; 4];
            stream.read_exact(&mut input).unwrap();
            assert_eq!(&input, b"ping");
            stream.write_all(b"pong").unwrap();
        });

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let sockaddr = 0x3000_0100;
        let send_buf = 0x3000_0200;
        let recv_buf = 0x3000_0300;
        memory.write_sockaddr_v4(sockaddr, Ipv4Addr::LOCALHOST, port);
        memory.write_bytes_at(send_buf, b"ping");
        let socket = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_STREAM, 0],
        );

        assert_ne!(socket, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("connect"),
                &mut memory,
                &[socket, sockaddr, 16],
            ),
            0
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("send"),
                &mut memory,
                &[socket, send_buf, 4, 0],
            ),
            4
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("recv"),
                &mut memory,
                &[socket, recv_buf, 4, 0],
            ),
            4
        );
        assert_eq!(
            (0..4)
                .map(|offset| memory.read_u8(recv_buf + offset).unwrap())
                .collect::<Vec<_>>(),
            b"pong"
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("closesocket"),
                &mut memory,
                &[socket],
            ),
            0
        );
        server.join().unwrap();
    }

    #[test]
    fn inet_and_byte_order_helpers_match_guest_layout() {
        reset_for_tests();
        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let text = 0x3000_1000;
        memory.write_bytes_at(text, b"127.0.0.1\0");

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("inet_addr"),
                &mut memory,
                &[text],
            ),
            u32::from_le_bytes([127, 0, 0, 1])
        );
        assert_eq!(
            dispatch_import(&mut kernel, 1, None, Some("htons"), &mut memory, &[0x1234],),
            0x3412
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("htonl"),
                &mut memory,
                &[0x1234_5678],
            ),
            0x7856_3412
        );
    }
}
