use crate::{
    ce::{coredll::CoredllGuestMemory, kernel::CeKernel, memory::PROCESS_HEAP_HANDLE},
    error::Result,
};

use std::{
    collections::{BTreeMap, VecDeque},
    io::{ErrorKind, Read, Write},
    net::{
        Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpListener, TcpStream, ToSocketAddrs,
        UdpSocket,
    },
    sync::{Arc, Mutex, OnceLock},
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
const SOL_SOCKET: u32 = 0xffff;
const SO_SNDTIMEO: u32 = 0x1005;
const SO_RCVTIMEO: u32 = 0x1006;
const SO_ERROR: u32 = 0x1007;
const MSG_OOB: u32 = 0x0001;
const FD_SETSIZE: u32 = 64;
const SOCKET_HANDLE_BASE: u32 = 0x7100_0000;
const MAX_GUEST_IO: usize = 1024 * 1024;
const CE_GATEWAY_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);
const CE_GUEST_IP: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 2);
const HOST_LOOPBACK_IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinsockNetworkMode {
    IsolatedNat {
        gateway: Ipv4Addr,
        guest_ip: Ipv4Addr,
    },
}

impl Default for WinsockNetworkMode {
    fn default() -> Self {
        Self::IsolatedNat {
            gateway: CE_GATEWAY_IP,
            guest_ip: CE_GUEST_IP,
        }
    }
}

const WSAEINTR: u32 = 10004;
const WSAEBADF: u32 = 10009;
const WSAEFAULT: u32 = 10014;
const WSAEINVAL: u32 = 10022;
const WSAEMFILE: u32 = 10024;
const WSAEWOULDBLOCK: u32 = 10035;
const WSAETIMEDOUT: u32 = 10060;
const WSAENOPROTOOPT: u32 = 10042;
const WSAEADDRINUSE: u32 = 10048;
const WSAECONNRESET: u32 = 10054;
const WSAENOTCONN: u32 = 10057;
const WSAECONNREFUSED: u32 = 10061;
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
            raw_import_arg(args, 3),
        ),
        (_, "recv" | "wsarecv") => recv_raw(
            kernel,
            thread_id,
            memory,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
            raw_import_arg(args, 3),
        ),
        (_, "sendto" | "wsasendto") => sendto_raw(
            kernel,
            thread_id,
            memory,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
            raw_import_arg(args, 3),
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
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
            raw_import_arg(args, 3),
            raw_import_arg(args, 4),
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
        (_, "setsockopt") => setsockopt_raw(
            kernel,
            thread_id,
            memory,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
            raw_import_arg(args, 3),
            raw_import_arg(args, 4),
        ),
        (_, "getsockopt") => getsockopt_raw(
            kernel,
            thread_id,
            memory,
            raw_import_arg(args, 0),
            raw_import_arg(args, 1),
            raw_import_arg(args, 2),
            raw_import_arg(args, 3),
            raw_import_arg(args, 4),
        ),
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
    TcpConnecting(Arc<Mutex<Option<std::result::Result<TcpStream, u32>>>>),
    TcpStream(TcpStream),
    TcpListener {
        listener: TcpListener,
        pending: VecDeque<(TcpStream, SocketAddr)>,
    },
    Udp(UdpSocket),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct SocketOptions {
    recv_timeout_ms: u32,
    send_timeout_ms: u32,
    nonblocking: bool,
    last_error: u32,
    oob_ready: bool,
}

#[derive(Default)]
struct WinsockState {
    next_socket: u32,
    sockets: BTreeMap<u32, HostSocket>,
    options: BTreeMap<u32, SocketOptions>,
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
                self.options.insert(handle, SocketOptions::default());
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

pub fn network_mode() -> WinsockNetworkMode {
    WinsockNetworkMode::default()
}

pub fn socket_read_ready(socket: u32) -> bool {
    let mut state = match state().lock() {
        Ok(state) => state,
        Err(_) => return false,
    };
    let options = state.options.get(&socket).copied().unwrap_or_default();
    let (ready, error) = match state.sockets.get_mut(&socket) {
        Some(HostSocket::TcpStream(stream)) => tcp_stream_read_ready(stream),
        Some(socket) => (probe_host_socket_read_ready(socket, options), None),
        None => (false, None),
    };
    if let Some(error) = error {
        record_socket_last_error(&mut state, socket, error);
    }
    ready
}

pub fn socket_read_wait_candidate(socket: u32) -> bool {
    state()
        .lock()
        .ok()
        .and_then(|state| state.sockets.get(&socket).map(host_socket_can_read_wait))
        .unwrap_or(false)
}

pub fn socket_accept_wait_candidate(socket: u32) -> bool {
    state()
        .lock()
        .ok()
        .and_then(|state| {
            state
                .sockets
                .get(&socket)
                .map(|socket| matches!(socket, HostSocket::TcpListener { .. }))
        })
        .unwrap_or(false)
}

pub fn socket_write_ready(socket: u32) -> bool {
    state()
        .lock()
        .ok()
        .and_then(|state| state.sockets.get(&socket).map(host_socket_write_ready))
        .unwrap_or(false)
}

pub fn socket_write_wait_candidate(socket: u32) -> bool {
    state()
        .lock()
        .ok()
        .and_then(|state| state.sockets.get(&socket).map(host_socket_can_write_wait))
        .unwrap_or(false)
}

pub fn socket_connect_wait_candidate(socket: u32) -> bool {
    state()
        .lock()
        .ok()
        .and_then(|state| {
            state.sockets.get(&socket).map(|s| {
                matches!(
                    s,
                    HostSocket::Pending {
                        family: AF_INET,
                        socket_type: SOCK_STREAM,
                        nonblocking: false,
                        ..
                    }
                )
            })
        })
        .unwrap_or(false)
}

pub fn socket_begin_tcp_connect(socket: u32, name_bytes: &[u8]) -> bool {
    if name_bytes.len() < 16 {
        return false;
    }
    let family = u16::from_le_bytes([name_bytes[0], name_bytes[1]]);
    if family as u32 != AF_INET {
        return false;
    }
    let port = u16::from_be_bytes([name_bytes[2], name_bytes[3]]);
    let ip = Ipv4Addr::new(name_bytes[4], name_bytes[5], name_bytes[6], name_bytes[7]);
    let host_addr = guest_to_host_addr(SocketAddrV4::new(ip, port));

    let mut state = match state().lock() {
        Ok(s) => s,
        Err(_) => return false,
    };
    let options = state.options.get(&socket).copied().unwrap_or_default();
    match state.sockets.get(&socket) {
        Some(HostSocket::Pending {
            family: AF_INET,
            socket_type: SOCK_STREAM,
            nonblocking: false,
            ..
        }) => {}
        _ => return false,
    }
    let result: Arc<Mutex<Option<std::result::Result<TcpStream, u32>>>> =
        Arc::new(Mutex::new(None));
    *state.sockets.get_mut(&socket).unwrap() = HostSocket::TcpConnecting(result.clone());
    drop(state);
    std::thread::spawn(move || {
        let r = TcpStream::connect_timeout(&host_addr, Duration::from_secs(30))
            .map(|stream| {
                configure_stream(&stream, options);
                stream
            })
            .map_err(|e| io_to_wsa_error(&e));
        if let Ok(mut lock) = result.lock() {
            *lock = Some(r);
        }
    });
    true
}

pub fn socket_except_wait_candidate(socket: u32) -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|state| state.sockets.contains_key(&socket))
}

pub fn socket_except_ready(socket: u32) -> bool {
    let mut state = match state().lock() {
        Ok(state) => state,
        Err(_) => return false,
    };
    if !state.sockets.contains_key(&socket) {
        return false;
    }
    if let Some(error) = state
        .sockets
        .get(&socket)
        .and_then(host_socket_take_error)
        .filter(|error| *error != 0)
    {
        set_socket_last_error(&mut state, socket, error);
    }
    state
        .options
        .get(&socket)
        .is_some_and(|options| options.last_error != 0 || options.oob_ready)
}

pub fn socket_recv_timeout_ms(socket: u32) -> Option<u32> {
    state()
        .lock()
        .ok()
        .and_then(|state| state.options.get(&socket).copied())
        .and_then(|options| (options.recv_timeout_ms != 0).then_some(options.recv_timeout_ms))
}

pub fn socket_nonblocking(socket: u32) -> bool {
    state()
        .lock()
        .ok()
        .and_then(|state| state.options.get(&socket).copied())
        .is_some_and(|options| options.nonblocking)
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
    let removed = state().lock().ok().and_then(|mut state| {
        state.options.remove(&socket);
        state.sockets.remove(&socket)
    });
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
    let options = state.options.get(&socket).copied().unwrap_or_default();
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
                    configure_stream(&stream, options);
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
                            apply_socket_options(entry, options);
                            Ok(())
                        }
                        Err(error) => Err(io_to_wsa_error(&error)),
                    }
                }
                Err(error) => Err(io_to_wsa_error(&error)),
            }
        }
        HostSocket::TcpConnecting(arc) => match arc.lock().ok().and_then(|mut lock| lock.take()) {
            Some(Ok(stream)) => {
                *entry = HostSocket::TcpStream(stream);
                Ok(())
            }
            Some(Err(error)) => Err(error),
            None => Err(WSAETIMEDOUT),
        },
        HostSocket::TcpStream(_) | HostSocket::Udp(_) => Ok(()),
        _ => Err(WSAEOPNOTSUPP),
    };
    match result {
        Ok(()) => {
            set_socket_last_error(&mut state, socket, 0);
            finish_socket_result(kernel, thread_id, Ok(()))
        }
        Err(error) => {
            record_socket_last_error(&mut state, socket, error);
            set_wsa_error(kernel, thread_id, error);
            SOCKET_ERROR
        }
    }
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
    let options = state.options.get(&socket).copied().unwrap_or_default();
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
                *entry = HostSocket::TcpListener {
                    listener,
                    pending: VecDeque::new(),
                };
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
                apply_socket_options(entry, options);
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
                .map(|socket| matches!(socket, HostSocket::TcpListener { .. }))
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
        let accepted = match state.sockets.get_mut(&socket) {
            Some(HostSocket::TcpListener { listener, pending }) => {
                if let Some(accepted) = pending.pop_front() {
                    accepted
                } else {
                    match listener.accept() {
                        Ok(accepted) => accepted,
                        Err(error) => {
                            set_wsa_error(kernel, thread_id, io_to_wsa_error(&error));
                            return INVALID_SOCKET;
                        }
                    }
                }
            }
            Some(_) | None => {
                set_wsa_error(kernel, thread_id, WSAEBADF);
                return INVALID_SOCKET;
            }
        };
        configure_stream(
            &accepted.0,
            SocketOptions {
                nonblocking: true,
                ..SocketOptions::default()
            },
        );
        let handle = match state.allocate(HostSocket::TcpStream(accepted.0)) {
            Some(handle) => handle,
            None => {
                set_wsa_error(kernel, thread_id, WSAEMFILE);
                return INVALID_SOCKET;
            }
        };
        (handle, accepted.1)
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
    flags: u32,
) -> u32 {
    let len = len.min(MAX_GUEST_IO as u32) as usize;
    let mut bytes = vec![0; len];
    if memory.read_bytes(buf_ptr, &mut bytes).is_err() {
        set_wsa_error(kernel, thread_id, WSAEFAULT);
        return SOCKET_ERROR;
    }
    let (result, sockets) = {
        let mut state = match state().lock() {
            Ok(state) => state,
            Err(_) => {
                set_wsa_error(kernel, thread_id, WSAEINTR);
                return SOCKET_ERROR;
            }
        };
        let oob_peer_handles = if flags & MSG_OOB != 0 {
            tcp_peer_socket_handles(&state, socket)
        } else {
            Vec::new()
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
        let sockets = (result.as_ref().ok().copied().unwrap_or(0) > 0)
            .then(|| state.sockets.keys().copied().collect::<Vec<_>>());
        match &result {
            Ok(count) => {
                set_socket_last_error(&mut state, socket, 0);
                if flags & MSG_OOB != 0 && *count != 0 {
                    for peer in oob_peer_handles {
                        if let Some(options) = state.options.get_mut(&peer) {
                            options.oob_ready = true;
                        }
                    }
                }
            }
            Err(error) => record_socket_last_error(&mut state, socket, io_to_wsa_error(error)),
        }
        (result, sockets)
    };
    if let Some(sockets) = sockets {
        kernel.queue_winsock_wake_candidates_for_handles(sockets);
    }
    finish_socket_count(kernel, thread_id, result)
}

fn recv_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    socket: u32,
    buf_ptr: u32,
    len: u32,
    flags: u32,
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
            set_socket_last_error(&mut state, socket, 0);
            if flags & MSG_OOB != 0
                && let Some(options) = state.options.get_mut(&socket)
            {
                options.oob_ready = false;
            }
            if memory.write_bytes(buf_ptr, &bytes[..count]).is_err() {
                set_wsa_error(kernel, thread_id, WSAEFAULT);
                SOCKET_ERROR
            } else {
                set_wsa_error(kernel, thread_id, 0);
                count as u32
            }
        }
        Err(error) => {
            let error = io_to_wsa_error(&error);
            record_socket_last_error(&mut state, socket, error);
            set_wsa_error(kernel, thread_id, error);
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
    flags: u32,
    to_ptr: u32,
    to_len: u32,
) -> u32 {
    if to_ptr == 0 {
        return send_raw(kernel, thread_id, memory, socket, buf_ptr, len, flags);
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
    let (result, sockets) = {
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
        let sockets = (result.as_ref().ok().copied().unwrap_or(0) > 0)
            .then(|| state.sockets.keys().copied().collect::<Vec<_>>());
        match &result {
            Ok(_) => set_socket_last_error(&mut state, socket, 0),
            Err(error) => record_socket_last_error(&mut state, socket, io_to_wsa_error(error)),
        }
        (result, sockets)
    };
    if let Some(sockets) = sockets {
        kernel.queue_winsock_wake_candidates_for_handles(sockets);
    }
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
            set_socket_last_error(&mut state, socket, 0);
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
            let error = io_to_wsa_error(&error);
            record_socket_last_error(&mut state, socket, error);
            set_wsa_error(kernel, thread_id, error);
            SOCKET_ERROR
        }
    }
}

fn select_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    _nfds: u32,
    readfds_ptr: u32,
    writefds_ptr: u32,
    exceptfds_ptr: u32,
    _timeout_ptr: u32,
) -> u32 {
    let readfds = match read_fd_set_sockets(memory, readfds_ptr) {
        Ok(fdset) => fdset,
        Err(error) => {
            set_wsa_error(kernel, thread_id, error);
            return SOCKET_ERROR;
        }
    };
    let writefds = match read_fd_set_sockets(memory, writefds_ptr) {
        Ok(fdset) => fdset,
        Err(error) => {
            set_wsa_error(kernel, thread_id, error);
            return SOCKET_ERROR;
        }
    };
    let exceptfds = match read_fd_set_sockets(memory, exceptfds_ptr) {
        Ok(fdset) => fdset,
        Err(error) => {
            set_wsa_error(kernel, thread_id, error);
            return SOCKET_ERROR;
        }
    };
    if readfds.is_none() && writefds.is_none() && exceptfds.is_none() {
        set_wsa_error(kernel, thread_id, WSAEINVAL);
        return SOCKET_ERROR;
    }
    if let Err(error) = validate_fd_set_sockets(&readfds, &writefds, &exceptfds) {
        set_wsa_error(kernel, thread_id, error);
        return SOCKET_ERROR;
    }

    let mut ready = 0;
    match filter_fd_set_by_socket(memory, readfds_ptr, socket_read_ready) {
        Ok(count) => ready += count,
        Err(_) => {
            set_wsa_error(kernel, thread_id, WSAEFAULT);
            return SOCKET_ERROR;
        }
    }
    match filter_fd_set(memory, writefds_ptr, |socket| {
        host_socket_write_ready(socket)
    }) {
        Ok(count) => ready += count,
        Err(_) => {
            set_wsa_error(kernel, thread_id, WSAEFAULT);
            return SOCKET_ERROR;
        }
    }
    match filter_fd_set_by_socket(memory, exceptfds_ptr, socket_except_ready) {
        Ok(count) => ready += count,
        Err(_) => {
            set_wsa_error(kernel, thread_id, WSAEFAULT);
            return SOCKET_ERROR;
        }
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
    for index in 0..count.min(FD_SETSIZE) {
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
    if !state.sockets.contains_key(&socket) {
        set_wsa_error(kernel, thread_id, WSAEBADF);
        return SOCKET_ERROR;
    };
    let options = state.options.entry(socket).or_default();
    options.nonblocking = nonblocking;
    let options = *options;
    let Some(entry) = state.sockets.get_mut(&socket) else {
        set_wsa_error(kernel, thread_id, WSAEBADF);
        return SOCKET_ERROR;
    };
    apply_socket_options(entry, options);
    set_wsa_error(kernel, thread_id, 0);
    0
}

fn setsockopt_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    socket: u32,
    level: u32,
    optname: u32,
    optval: u32,
    optlen: u32,
) -> u32 {
    if optval == 0 || optlen < 4 {
        set_wsa_error(kernel, thread_id, WSAEFAULT);
        return SOCKET_ERROR;
    }
    if level != SOL_SOCKET || !matches!(optname, SO_RCVTIMEO | SO_SNDTIMEO) {
        set_wsa_error(kernel, thread_id, WSAENOPROTOOPT);
        return SOCKET_ERROR;
    }
    let timeout_ms = match memory.read_u32(optval) {
        Ok(timeout_ms) => timeout_ms,
        Err(_) => {
            set_wsa_error(kernel, thread_id, WSAEFAULT);
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
    if !state.sockets.contains_key(&socket) {
        set_wsa_error(kernel, thread_id, WSAEBADF);
        return SOCKET_ERROR;
    }
    let options = state.options.entry(socket).or_default();
    match optname {
        SO_RCVTIMEO => options.recv_timeout_ms = timeout_ms,
        SO_SNDTIMEO => options.send_timeout_ms = timeout_ms,
        _ => unreachable!(),
    }
    let options = *options;
    if let Some(entry) = state.sockets.get_mut(&socket) {
        apply_socket_options(entry, options);
    }
    set_wsa_error(kernel, thread_id, 0);
    0
}

fn getsockopt_raw<M: CoredllGuestMemory>(
    kernel: &mut CeKernel,
    thread_id: u32,
    memory: &mut M,
    socket: u32,
    level: u32,
    optname: u32,
    optval: u32,
    optlen_ptr: u32,
) -> u32 {
    if optval == 0 || optlen_ptr == 0 {
        set_wsa_error(kernel, thread_id, WSAEFAULT);
        return SOCKET_ERROR;
    }
    if level != SOL_SOCKET || !matches!(optname, SO_RCVTIMEO | SO_SNDTIMEO | SO_ERROR) {
        set_wsa_error(kernel, thread_id, WSAENOPROTOOPT);
        return SOCKET_ERROR;
    }
    let optlen = match memory.read_u32(optlen_ptr) {
        Ok(optlen) => optlen,
        Err(_) => {
            set_wsa_error(kernel, thread_id, WSAEFAULT);
            return SOCKET_ERROR;
        }
    };
    if optlen < 4 {
        set_wsa_error(kernel, thread_id, WSAEFAULT);
        return SOCKET_ERROR;
    }
    let value = {
        let mut state = match state().lock() {
            Ok(state) => state,
            Err(_) => {
                set_wsa_error(kernel, thread_id, WSAEINTR);
                return SOCKET_ERROR;
            }
        };
        if !state.sockets.contains_key(&socket) {
            set_wsa_error(kernel, thread_id, WSAEBADF);
            return SOCKET_ERROR;
        }
        if let Some(error) = state
            .sockets
            .get(&socket)
            .and_then(host_socket_take_error)
            .filter(|error| *error != 0)
        {
            set_socket_last_error(&mut state, socket, error);
        }
        let options = state.options.get(&socket).copied().unwrap_or_default();
        match optname {
            SO_RCVTIMEO => options.recv_timeout_ms,
            SO_SNDTIMEO => options.send_timeout_ms,
            SO_ERROR => {
                let error = options.last_error;
                set_socket_last_error(&mut state, socket, 0);
                error
            }
            _ => unreachable!(),
        }
    };
    if memory.write_u32(optval, value).is_err() || memory.write_u32(optlen_ptr, 4).is_err() {
        set_wsa_error(kernel, thread_id, WSAEFAULT);
        return SOCKET_ERROR;
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
    let addr = if name.eq_ignore_ascii_case("fakece") {
        CE_GUEST_IP
    } else if name.eq_ignore_ascii_case("gateway") {
        CE_GATEWAY_IP
    } else if name.eq_ignore_ascii_case("localhost") {
        CE_GATEWAY_IP
    } else if let Ok(ip) = name.parse::<Ipv4Addr>() {
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

fn filter_fd_set<M, F>(memory: &mut M, fdset_ptr: u32, mut ready_fn: F) -> Result<u32>
where
    M: CoredllGuestMemory,
    F: FnMut(&mut HostSocket) -> bool,
{
    if fdset_ptr == 0 {
        return Ok(0);
    }
    let count = memory.read_u32(fdset_ptr)?.min(FD_SETSIZE);
    let mut ready_sockets = Vec::new();
    let mut state = state().lock().ok();
    for index in 0..count {
        let socket = memory.read_u32(fdset_ptr + 4 + index * 4)?;
        if state
            .as_mut()
            .and_then(|state| state.sockets.get_mut(&socket))
            .is_some_and(&mut ready_fn)
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

fn read_fd_set_sockets<M: CoredllGuestMemory>(
    memory: &mut M,
    fdset_ptr: u32,
) -> std::result::Result<Option<Vec<u32>>, u32> {
    if fdset_ptr == 0 {
        return Ok(None);
    }
    let count = memory.read_u32(fdset_ptr).map_err(|_| WSAEFAULT)?;
    if count > FD_SETSIZE {
        return Err(WSAEINVAL);
    }
    let mut sockets = Vec::with_capacity(count as usize);
    for index in 0..count {
        sockets.push(
            memory
                .read_u32(fdset_ptr + 4 + index * 4)
                .map_err(|_| WSAEFAULT)?,
        );
    }
    Ok(Some(sockets))
}

fn validate_fd_set_sockets(
    readfds: &Option<Vec<u32>>,
    writefds: &Option<Vec<u32>>,
    exceptfds: &Option<Vec<u32>>,
) -> std::result::Result<(), u32> {
    let state = state().lock().map_err(|_| WSAEINTR)?;
    for socket in readfds
        .iter()
        .chain(writefds.iter())
        .chain(exceptfds.iter())
        .flat_map(|sockets| sockets.iter().copied())
    {
        if !state.sockets.contains_key(&socket) {
            return Err(WSAEBADF);
        }
    }
    Ok(())
}

fn filter_fd_set_by_socket<M, F>(memory: &mut M, fdset_ptr: u32, mut ready_fn: F) -> Result<u32>
where
    M: CoredllGuestMemory,
    F: FnMut(u32) -> bool,
{
    if fdset_ptr == 0 {
        return Ok(0);
    }
    let count = memory.read_u32(fdset_ptr)?.min(FD_SETSIZE);
    let mut ready_sockets = Vec::new();
    for index in 0..count {
        let socket = memory.read_u32(fdset_ptr + 4 + index * 4)?;
        if ready_fn(socket) {
            ready_sockets.push(socket);
        }
    }
    memory.write_u32(fdset_ptr, ready_sockets.len() as u32)?;
    for (index, socket) in ready_sockets.iter().copied().enumerate() {
        memory.write_u32(fdset_ptr + 4 + index as u32 * 4, socket)?;
    }
    Ok(ready_sockets.len() as u32)
}

fn host_socket_read_ready(socket: &HostSocket, options: SocketOptions) -> bool {
    match socket {
        HostSocket::TcpStream(stream) => tcp_stream_read_ready(stream).0,
        HostSocket::Udp(udp) => udp_socket_read_ready(udp, options),
        HostSocket::TcpListener { pending, .. } => !pending.is_empty(),
        HostSocket::Pending { .. } | HostSocket::TcpConnecting(_) => false,
    }
}

fn tcp_stream_read_ready(stream: &TcpStream) -> (bool, Option<u32>) {
    let mut byte = [0; 1];
    match stream.peek(&mut byte) {
        Ok(_) => (true, None),
        Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
            (false, None)
        }
        Err(error) => (true, Some(io_to_wsa_error(&error))),
    }
}

fn host_socket_can_read_wait(socket: &HostSocket) -> bool {
    matches!(
        socket,
        HostSocket::TcpStream(_) | HostSocket::TcpListener { .. } | HostSocket::Udp(_)
    )
}

fn host_socket_can_write_wait(socket: &HostSocket) -> bool {
    matches!(
        socket,
        HostSocket::TcpStream(_)
            | HostSocket::Udp(_)
            | HostSocket::TcpConnecting(_)
            | HostSocket::Pending {
                family: AF_INET,
                socket_type: SOCK_STREAM,
                ..
            }
    )
}

fn tcp_peer_socket_handles(state: &WinsockState, socket: u32) -> Vec<u32> {
    let Some(HostSocket::TcpStream(stream)) = state.sockets.get(&socket) else {
        return Vec::new();
    };
    let Ok(local) = stream.local_addr() else {
        return Vec::new();
    };
    let Ok(peer) = stream.peer_addr() else {
        return Vec::new();
    };
    state
        .sockets
        .iter()
        .filter_map(|(handle, candidate)| {
            if *handle == socket {
                return None;
            }
            let HostSocket::TcpStream(candidate) = candidate else {
                return None;
            };
            let candidate_local = candidate.local_addr().ok()?;
            let candidate_peer = candidate.peer_addr().ok()?;
            (candidate_local == peer && candidate_peer == local).then_some(*handle)
        })
        .collect()
}

fn probe_host_socket_read_ready(socket: &mut HostSocket, options: SocketOptions) -> bool {
    match socket {
        HostSocket::TcpListener { listener, pending } => {
            if !pending.is_empty() {
                return true;
            }
            match listener.accept() {
                Ok((stream, remote)) => {
                    configure_stream(
                        &stream,
                        SocketOptions {
                            nonblocking: true,
                            ..SocketOptions::default()
                        },
                    );
                    pending.push_back((stream, remote));
                    true
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => false,
                Err(_) => false,
            }
        }
        socket => host_socket_read_ready(socket, options),
    }
}

fn udp_socket_read_ready(udp: &UdpSocket, options: SocketOptions) -> bool {
    let mut bytes = [0; 2048];
    if options.nonblocking {
        return udp.peek_from(&mut bytes).is_ok();
    }
    let _ = udp.set_nonblocking(true);
    let ready = udp.peek_from(&mut bytes).is_ok();
    let _ = udp.set_nonblocking(false);
    ready
}

fn host_socket_write_ready(socket: &HostSocket) -> bool {
    match socket {
        HostSocket::TcpStream(_) | HostSocket::Udp(_) => true,
        HostSocket::TcpConnecting(arc) => arc.lock().ok().is_some_and(|lock| lock.is_some()),
        _ => false,
    }
}

fn host_socket_take_error(socket: &HostSocket) -> Option<u32> {
    let error = match socket {
        HostSocket::TcpStream(stream) => stream.take_error().ok().flatten(),
        HostSocket::TcpListener { listener, .. } => listener.take_error().ok().flatten(),
        HostSocket::Udp(udp) => udp.take_error().ok().flatten(),
        HostSocket::Pending { .. } | HostSocket::TcpConnecting(_) => None,
    }?;
    Some(io_to_wsa_error(&error))
}

fn set_socket_last_error(state: &mut WinsockState, socket: u32, error: u32) {
    if let Some(options) = state.options.get_mut(&socket) {
        options.last_error = error;
    }
}

fn record_socket_last_error(state: &mut WinsockState, socket: u32, error: u32) {
    if error != WSAEWOULDBLOCK {
        set_socket_last_error(state, socket, error);
    }
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
    Ok(guest_to_host_addr(SocketAddrV4::new(ip, port)))
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
    let SocketAddr::V4(addr) = host_to_guest_addr(addr) else {
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

fn guest_to_host_addr(addr: SocketAddrV4) -> SocketAddr {
    let ip = *addr.ip();
    let translated = if ip == CE_GATEWAY_IP || ip == CE_GUEST_IP {
        HOST_LOOPBACK_IP
    } else if ip == Ipv4Addr::UNSPECIFIED {
        Ipv4Addr::UNSPECIFIED
    } else {
        ip
    };
    SocketAddr::V4(SocketAddrV4::new(translated, addr.port()))
}

fn host_to_guest_addr(addr: SocketAddr) -> SocketAddr {
    let SocketAddr::V4(addr) = addr else {
        return addr;
    };
    let ip = *addr.ip();
    let translated = if ip == HOST_LOOPBACK_IP || ip == Ipv4Addr::UNSPECIFIED {
        CE_GATEWAY_IP
    } else {
        ip
    };
    SocketAddr::V4(SocketAddrV4::new(translated, addr.port()))
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

fn apply_socket_options(socket: &mut HostSocket, options: SocketOptions) {
    match socket {
        HostSocket::Pending { nonblocking, .. } => *nonblocking = options.nonblocking,
        HostSocket::TcpConnecting(_) => {}
        HostSocket::TcpStream(stream) => configure_stream(stream, options),
        HostSocket::TcpListener { listener, .. } => {
            let _ = listener.set_nonblocking(options.nonblocking);
        }
        HostSocket::Udp(udp) => {
            let _ = udp.set_nonblocking(options.nonblocking);
            if !options.nonblocking {
                let _ = udp.set_read_timeout(Some(socket_timeout_duration(
                    options.recv_timeout_ms,
                    Duration::from_millis(1),
                )));
                let _ = udp.set_write_timeout(Some(socket_timeout_duration(
                    options.send_timeout_ms,
                    Duration::from_secs(3),
                )));
            }
        }
    }
}

fn configure_stream(stream: &TcpStream, options: SocketOptions) {
    let _ = stream.set_nonblocking(options.nonblocking);
    if !options.nonblocking {
        let _ = stream.set_read_timeout(Some(socket_timeout_duration(
            options.recv_timeout_ms,
            Duration::from_millis(1),
        )));
        let _ = stream.set_write_timeout(Some(socket_timeout_duration(
            options.send_timeout_ms,
            Duration::from_secs(3),
        )));
    }
}

fn socket_timeout_duration(timeout_ms: u32, fallback: Duration) -> Duration {
    if timeout_ms == 0 {
        fallback
    } else {
        Duration::from_millis(u64::from(timeout_ms))
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
        ErrorKind::ConnectionRefused => WSAECONNREFUSED,
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
pub(crate) fn reset_for_tests() {
    *state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = WinsockState::default();
}

#[cfg(test)]
pub(crate) fn locked_reset_for_tests() -> std::sync::MutexGuard<'static, ()> {
    static WINSOCK_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = WINSOCK_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    reset_for_tests();
    guard
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Result,
        ce::{scheduler::SchedulerBlockedWaitKind, timer::INFINITE},
        config::RuntimeConfig,
        error::Error,
    };
    #[cfg(windows)]
    use std::os::windows::io::AsRawSocket;
    use std::{
        collections::BTreeMap,
        io::Read,
        net::{TcpListener, UdpSocket},
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };

    #[cfg(windows)]
    const SO_LINGER: i32 = 0x0080;

    #[cfg(windows)]
    #[repr(C)]
    struct HostLinger {
        l_onoff: u16,
        l_linger: u16,
    }

    #[cfg(windows)]
    unsafe extern "system" {
        fn setsockopt(s: usize, level: i32, optname: i32, optval: *const i8, optlen: i32) -> i32;
    }

    #[cfg(windows)]
    fn force_tcp_reset_on_drop(stream: &TcpStream) {
        let linger = HostLinger {
            l_onoff: 1,
            l_linger: 0,
        };
        let result = unsafe {
            setsockopt(
                stream.as_raw_socket() as usize,
                SOL_SOCKET as i32,
                SO_LINGER,
                &linger as *const HostLinger as *const i8,
                std::mem::size_of::<HostLinger>() as i32,
            )
        };
        assert_eq!(
            result,
            0,
            "SO_LINGER reset setup failed: {}",
            std::io::Error::last_os_error()
        );
    }

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

        fn write_fd_set(&mut self, addr: u32, sockets: &[u32]) {
            self.write_u32(addr, sockets.len() as u32).unwrap();
            for (index, socket) in sockets.iter().copied().enumerate() {
                self.write_u32(addr + 4 + index as u32 * 4, socket).unwrap();
            }
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

    #[derive(Default)]
    struct FaultingTestMemory {
        inner: TestMemory,
        read_faults: Vec<u32>,
        write_faults: Vec<u32>,
    }

    impl FaultingTestMemory {
        fn fail() -> Error {
            Error::Backend("test memory fault".to_owned())
        }
    }

    impl CoredllGuestMemory for FaultingTestMemory {
        fn read_u8(&self, addr: u32) -> Result<u8> {
            if self.read_faults.contains(&addr) {
                return Err(Self::fail());
            }
            self.inner.read_u8(addr)
        }

        fn write_u8(&mut self, addr: u32, value: u8) -> Result<()> {
            if self.write_faults.contains(&addr) {
                return Err(Self::fail());
            }
            self.inner.write_u8(addr, value)
        }

        fn read_u32(&self, addr: u32) -> Result<u32> {
            if self.read_faults.contains(&addr) {
                return Err(Self::fail());
            }
            self.inner.read_u32(addr)
        }

        fn write_u32(&mut self, addr: u32, value: u32) -> Result<()> {
            if self.write_faults.contains(&addr) {
                return Err(Self::fail());
            }
            self.inner.write_u32(addr, value)
        }

        fn read_u16(&self, addr: u32) -> Result<u16> {
            if self.read_faults.contains(&addr) {
                return Err(Self::fail());
            }
            self.inner.read_u16(addr)
        }

        fn write_u16(&mut self, addr: u32, value: u16) -> Result<()> {
            if self.write_faults.contains(&addr) {
                return Err(Self::fail());
            }
            self.inner.write_u16(addr, value)
        }
    }

    fn test_kernel() -> CeKernel {
        CeKernel::boot(RuntimeConfig::load("regs.json", "serial_devices.json").unwrap())
    }

    fn host_socket_timeouts(socket: u32) -> Option<(Option<Duration>, Option<Duration>)> {
        let state = state().lock().unwrap();
        match state.sockets.get(&socket)? {
            HostSocket::TcpStream(stream) => Some((
                stream.read_timeout().unwrap(),
                stream.write_timeout().unwrap(),
            )),
            HostSocket::Udp(udp) => {
                Some((udp.read_timeout().unwrap(), udp.write_timeout().unwrap()))
            }
            _ => None,
        }
    }

    #[test]
    fn wsa_startup_writes_ce_shaped_wsadata() {
        let _winsock_guard = locked_reset_for_tests();
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
        let _winsock_guard = locked_reset_for_tests();
        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();

        assert_eq!(
            dispatch_import(&mut kernel, 1, Some(3), None, &mut memory, &[0x0202, 0]),
            WSAVERNOTSUPPORTED
        );
    }

    #[test]
    fn host_tcp_socket_connect_send_and_recv_use_loopback() {
        let _winsock_guard = locked_reset_for_tests();
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
    fn socket_timeouts_round_trip_through_setsockopt_getsockopt() {
        let _winsock_guard = locked_reset_for_tests();
        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let optval = 0x3000_5100;
        let optlen = 0x3000_5200;
        let socket = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_STREAM, 0],
        );
        assert_ne!(socket, INVALID_SOCKET);

        memory.write_u32(optval, 250).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("setsockopt"),
                &mut memory,
                &[socket, SOL_SOCKET, SO_RCVTIMEO, optval, 4],
            ),
            0
        );
        assert_eq!(socket_recv_timeout_ms(socket), Some(250));

        memory.write_u32(optval, 0).unwrap();
        memory.write_u32(optlen, 4).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("getsockopt"),
                &mut memory,
                &[socket, SOL_SOCKET, SO_RCVTIMEO, optval, optlen],
            ),
            0
        );
        assert_eq!(memory.read_u32(optval).unwrap(), 250);
        assert_eq!(memory.read_u32(optlen).unwrap(), 4);

        memory.write_u32(optval, 750).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("setsockopt"),
                &mut memory,
                &[socket, SOL_SOCKET, SO_SNDTIMEO, optval, 4],
            ),
            0
        );
        memory.write_u32(optval, 0).unwrap();
        memory.write_u32(optlen, 4).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("getsockopt"),
                &mut memory,
                &[socket, SOL_SOCKET, SO_SNDTIMEO, optval, optlen],
            ),
            0
        );
        assert_eq!(memory.read_u32(optval).unwrap(), 750);

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("setsockopt"),
                &mut memory,
                &[socket, SOL_SOCKET, 0xdead, optval, 4],
            ),
            SOCKET_ERROR
        );
        assert_eq!(kernel.threads.get_last_error(1), WSAENOPROTOOPT);
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
        assert_eq!(socket_recv_timeout_ms(socket), None);
    }

    #[test]
    fn socket_timeouts_apply_to_host_tcp_and_udp_sockets() {
        let _winsock_guard = locked_reset_for_tests();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let tcp_port = listener.local_addr().unwrap().port();
        let (accepted_tx, accepted_rx) = mpsc::channel();
        let (close_tx, close_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (_stream, _) = listener.accept().unwrap();
            accepted_tx.send(()).unwrap();
            close_rx.recv().unwrap();
        });
        let udp_probe = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let udp_port = udp_probe.local_addr().unwrap().port();
        drop(udp_probe);

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let tcp_addr = 0x3000_8600;
        let udp_addr = 0x3000_8700;
        let optval = 0x3000_8800;
        memory.write_sockaddr_v4(tcp_addr, Ipv4Addr::LOCALHOST, tcp_port);
        memory.write_sockaddr_v4(udp_addr, Ipv4Addr::LOCALHOST, udp_port);

        let tcp = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_STREAM, 0],
        );
        assert_ne!(tcp, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("connect"),
                &mut memory,
                &[tcp, tcp_addr, 16],
            ),
            0
        );
        accepted_rx.recv().unwrap();

        memory.write_u32(optval, 125).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("setsockopt"),
                &mut memory,
                &[tcp, SOL_SOCKET, SO_RCVTIMEO, optval, 4],
            ),
            0
        );
        memory.write_u32(optval, 375).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("setsockopt"),
                &mut memory,
                &[tcp, SOL_SOCKET, SO_SNDTIMEO, optval, 4],
            ),
            0
        );
        assert_eq!(
            host_socket_timeouts(tcp),
            Some((
                Some(Duration::from_millis(125)),
                Some(Duration::from_millis(375))
            ))
        );

        let udp = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_DGRAM, 0],
        );
        assert_ne!(udp, INVALID_SOCKET);
        memory.write_u32(optval, 222).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("setsockopt"),
                &mut memory,
                &[udp, SOL_SOCKET, SO_RCVTIMEO, optval, 4],
            ),
            0
        );
        memory.write_u32(optval, 444).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("setsockopt"),
                &mut memory,
                &[udp, SOL_SOCKET, SO_SNDTIMEO, optval, 4],
            ),
            0
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("bind"),
                &mut memory,
                &[udp, udp_addr, 16],
            ),
            0
        );
        assert_eq!(
            host_socket_timeouts(udp),
            Some((
                Some(Duration::from_millis(222)),
                Some(Duration::from_millis(444))
            ))
        );

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("closesocket"),
                &mut memory,
                &[udp],
            ),
            0
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("closesocket"),
                &mut memory,
                &[tcp],
            ),
            0
        );
        close_tx.send(()).unwrap();
        server.join().unwrap();
    }

    #[test]
    fn so_error_tracks_failed_connect_and_drives_exception_select() {
        let _winsock_guard = locked_reset_for_tests();
        let probe = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let sockaddr = 0x3000_8100;
        let optval = 0x3000_8200;
        let optlen = 0x3000_8300;
        let exceptfds = 0x3000_8400;
        memory.write_sockaddr_v4(sockaddr, Ipv4Addr::LOCALHOST, port);
        let socket = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_STREAM, 0],
        );
        assert_ne!(socket, INVALID_SOCKET);

        memory.write_u32(optval, u32::MAX).unwrap();
        memory.write_u32(optlen, 4).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("getsockopt"),
                &mut memory,
                &[socket, SOL_SOCKET, SO_ERROR, optval, optlen],
            ),
            0
        );
        assert_eq!(memory.read_u32(optval).unwrap(), 0);

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("connect"),
                &mut memory,
                &[socket, sockaddr, 16],
            ),
            SOCKET_ERROR
        );
        assert_eq!(kernel.threads.get_last_error(1), WSAECONNREFUSED);
        assert!(socket_except_ready(socket));

        memory.write_u32(exceptfds, 1).unwrap();
        memory.write_u32(exceptfds + 4, socket).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, 0, 0, exceptfds, 0],
            ),
            1
        );
        assert_eq!(memory.read_u32(exceptfds).unwrap(), 1);
        assert_eq!(memory.read_u32(exceptfds + 4).unwrap(), socket);

        memory.write_u32(optval, 0).unwrap();
        memory.write_u32(optlen, 4).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("getsockopt"),
                &mut memory,
                &[socket, SOL_SOCKET, SO_ERROR, optval, optlen],
            ),
            0
        );
        assert_eq!(memory.read_u32(optval).unwrap(), WSAECONNREFUSED);
        assert_eq!(memory.read_u32(optlen).unwrap(), 4);
        assert!(!socket_except_ready(socket));

        memory.write_u32(exceptfds, 1).unwrap();
        memory.write_u32(exceptfds + 4, socket).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, 0, 0, exceptfds, 0],
            ),
            0
        );
        assert_eq!(memory.read_u32(exceptfds).unwrap(), 0);
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
    }

    #[test]
    fn oob_send_marks_peer_exception_select_without_so_error() {
        let _winsock_guard = locked_reset_for_tests();
        let probe = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let sockaddr = 0x3000_8500;
        let readfds = 0x3000_8600;
        let exceptfds = 0x3000_8700;
        let optval = 0x3000_8800;
        let optlen = 0x3000_8900;
        let remote_addr = 0x3000_8a00;
        let remote_len = 0x3000_8b00;
        let send_buf = 0x3000_8c00;
        let recv_buf = 0x3000_8d00;
        memory.write_sockaddr_v4(sockaddr, Ipv4Addr::LOCALHOST, port);
        memory.write_bytes_at(send_buf, b"!");

        let listener = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_STREAM, 0],
        );
        assert_ne!(listener, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("bind"),
                &mut memory,
                &[listener, sockaddr, 16],
            ),
            0
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("listen"),
                &mut memory,
                &[listener, 1],
            ),
            0
        );

        let client = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_STREAM, 0],
        );
        assert_ne!(client, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("connect"),
                &mut memory,
                &[client, sockaddr, 16],
            ),
            0
        );

        memory.write_u32(readfds, 1).unwrap();
        memory.write_u32(readfds + 4, listener).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, readfds, 0, 0, 0],
            ),
            1
        );
        memory.write_u32(remote_len, 16).unwrap();
        let accepted = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("accept"),
            &mut memory,
            &[listener, remote_addr, remote_len],
        );
        assert_ne!(accepted, INVALID_SOCKET);
        assert!(!socket_except_ready(accepted));

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("send"),
                &mut memory,
                &[client, send_buf, 1, MSG_OOB],
            ),
            1
        );
        assert!(socket_except_ready(accepted));
        memory.write_u32(exceptfds, 1).unwrap();
        memory.write_u32(exceptfds + 4, accepted).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, 0, 0, exceptfds, 0],
            ),
            1
        );
        assert_eq!(memory.read_u32(exceptfds).unwrap(), 1);
        assert_eq!(memory.read_u32(exceptfds + 4).unwrap(), accepted);

        memory.write_u32(optval, u32::MAX).unwrap();
        memory.write_u32(optlen, 4).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("getsockopt"),
                &mut memory,
                &[accepted, SOL_SOCKET, SO_ERROR, optval, optlen],
            ),
            0
        );
        assert_eq!(memory.read_u32(optval).unwrap(), 0);
        assert!(socket_except_ready(accepted));

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("recv"),
                &mut memory,
                &[accepted, recv_buf, 1, MSG_OOB],
            ),
            1
        );
        assert_eq!(memory.read_u8(recv_buf).unwrap(), b'!');
        assert!(!socket_except_ready(accepted));
        memory.write_u32(exceptfds, 1).unwrap();
        memory.write_u32(exceptfds + 4, accepted).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, 0, 0, exceptfds, 0],
            ),
            0
        );
        assert_eq!(memory.read_u32(exceptfds).unwrap(), 0);

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("closesocket"),
                &mut memory,
                &[accepted],
            ),
            0
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("closesocket"),
                &mut memory,
                &[client],
            ),
            0
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("closesocket"),
                &mut memory,
                &[listener],
            ),
            0
        );
    }

    #[test]
    fn nonblocking_recv_without_data_returns_wouldblock_without_waiting() {
        let _winsock_guard = locked_reset_for_tests();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let (accepted_tx, accepted_rx) = mpsc::channel();
        let (close_tx, close_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (_stream, _) = listener.accept().unwrap();
            accepted_tx.send(()).unwrap();
            close_rx.recv().unwrap();
        });

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let sockaddr = 0x3000_6100;
        let nonblocking_ptr = 0x3000_6200;
        let recv_buf = 0x3000_6300;
        let optval = 0x3000_6400;
        let optlen = 0x3000_6500;
        memory.write_sockaddr_v4(sockaddr, Ipv4Addr::LOCALHOST, port);
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
        accepted_rx.recv().unwrap();
        memory.write_u32(nonblocking_ptr, 1).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("ioctlsocket"),
                &mut memory,
                &[socket, FIONBIO, nonblocking_ptr],
            ),
            0
        );
        assert!(socket_nonblocking(socket));
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("recv"),
                &mut memory,
                &[socket, recv_buf, 4, 0],
            ),
            SOCKET_ERROR
        );
        assert_eq!(kernel.threads.get_last_error(1), WSAEWOULDBLOCK);
        assert!(!socket_except_ready(socket));
        memory.write_u32(optval, u32::MAX).unwrap();
        memory.write_u32(optlen, 4).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("getsockopt"),
                &mut memory,
                &[socket, SOL_SOCKET, SO_ERROR, optval, optlen],
            ),
            0
        );
        assert_eq!(memory.read_u32(optval).unwrap(), 0);
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
        close_tx.send(()).unwrap();
        server.join().unwrap();
    }

    #[test]
    fn nonblocking_recvfrom_without_datagram_returns_wouldblock_without_waiting() {
        let _winsock_guard = locked_reset_for_tests();
        let probe = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let bind_addr = 0x3000_a100;
        let nonblocking_ptr = 0x3000_a200;
        let recv_buf = 0x3000_a300;
        let from_addr = 0x3000_a400;
        let from_len = 0x3000_a500;
        let optval = 0x3000_a600;
        let optlen = 0x3000_a700;
        memory.write_sockaddr_v4(bind_addr, Ipv4Addr::LOCALHOST, port);
        memory.write_u32(from_len, 16).unwrap();
        let socket = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_DGRAM, 0],
        );
        assert_ne!(socket, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("bind"),
                &mut memory,
                &[socket, bind_addr, 16],
            ),
            0
        );
        memory.write_u32(nonblocking_ptr, 1).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("ioctlsocket"),
                &mut memory,
                &[socket, FIONBIO, nonblocking_ptr],
            ),
            0
        );
        assert!(socket_nonblocking(socket));
        assert!(!socket_read_ready(socket));
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("recvfrom"),
                &mut memory,
                &[socket, recv_buf, 4, 0, from_addr, from_len],
            ),
            SOCKET_ERROR
        );
        assert_eq!(kernel.threads.get_last_error(1), WSAEWOULDBLOCK);
        assert!(!socket_except_ready(socket));
        memory.write_u32(optval, u32::MAX).unwrap();
        memory.write_u32(optlen, 4).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("getsockopt"),
                &mut memory,
                &[socket, SOL_SOCKET, SO_ERROR, optval, optlen],
            ),
            0
        );
        assert_eq!(memory.read_u32(optval).unwrap(), 0);
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
    }

    #[test]
    fn nonblocking_accept_without_client_returns_wouldblock_without_waiting() {
        let _winsock_guard = locked_reset_for_tests();
        let probe = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let bind_addr = 0x3000_7100;
        let nonblocking_ptr = 0x3000_7200;
        let remote_addr = 0x3000_7300;
        let remote_len = 0x3000_7400;
        memory.write_sockaddr_v4(bind_addr, Ipv4Addr::LOCALHOST, port);
        memory.write_u32(remote_len, 16).unwrap();
        let listener = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_STREAM, 0],
        );
        assert_ne!(listener, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("bind"),
                &mut memory,
                &[listener, bind_addr, 16],
            ),
            0
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("listen"),
                &mut memory,
                &[listener, 1],
            ),
            0
        );
        memory.write_u32(nonblocking_ptr, 1).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("ioctlsocket"),
                &mut memory,
                &[listener, FIONBIO, nonblocking_ptr],
            ),
            0
        );
        assert!(socket_nonblocking(listener));
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("accept"),
                &mut memory,
                &[listener, remote_addr, remote_len],
            ),
            INVALID_SOCKET
        );
        assert_eq!(kernel.threads.get_last_error(1), WSAEWOULDBLOCK);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("closesocket"),
                &mut memory,
                &[listener],
            ),
            0
        );
    }

    #[test]
    fn scheduler_waiter_can_use_socket_read_readiness() {
        let _winsock_guard = locked_reset_for_tests();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let (accepted_tx, accepted_rx) = mpsc::channel();
        let (write_tx, write_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            accepted_tx.send(()).unwrap();
            write_rx.recv().unwrap();
            stream.write_all(b"wake").unwrap();
        });

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let sockaddr = 0x3000_4100;
        let recv_buf = 0x3000_4200;
        memory.write_sockaddr_v4(sockaddr, Ipv4Addr::LOCALHOST, port);
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
        accepted_rx.recv().unwrap();
        assert!(!socket_read_ready(socket));

        let wait_id = kernel.register_blocked_waiter(
            70,
            0x770,
            vec![socket],
            SchedulerBlockedWaitKind::Kernel,
            0,
            INFINITE,
        );
        assert_eq!(kernel.queue_winsock_wake_candidates(socket), 1);
        assert_eq!(
            kernel.select_ready_blocked_waiter(1, 0, |blocked, _kernel| {
                blocked.wait_handles.iter().copied().any(socket_read_ready)
            }),
            None
        );

        write_tx.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        while !socket_read_ready(socket) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(socket_read_ready(socket));
        assert_eq!(kernel.queue_winsock_wake_candidates(socket), 0);
        assert_eq!(
            kernel.select_ready_blocked_waiter(1, 0, |blocked, _kernel| {
                blocked.wait_handles.iter().copied().any(socket_read_ready)
            }),
            Some(wait_id)
        );
        kernel.remove_blocked_waiter(wait_id).unwrap();
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
            b"wake"
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
    fn select_buffers_tcp_listener_readiness_for_accept() {
        let _winsock_guard = locked_reset_for_tests();
        let probe = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let bind_addr = 0x3000_5100;
        let readfds = 0x3000_5200;
        let remote_addr = 0x3000_5300;
        let remote_len = 0x3000_5400;
        memory.write_sockaddr_v4(bind_addr, Ipv4Addr::LOCALHOST, port);
        let listener = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_STREAM, 0],
        );
        assert_ne!(listener, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("bind"),
                &mut memory,
                &[listener, bind_addr, 16],
            ),
            0
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("listen"),
                &mut memory,
                &[listener, 1],
            ),
            0
        );

        memory.write_u32(readfds, 1).unwrap();
        memory.write_u32(readfds + 4, listener).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, readfds, 0, 0, 0],
            ),
            0
        );
        assert_eq!(memory.read_u32(readfds).unwrap(), 0);

        let client = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
        memory.write_u32(readfds, 1).unwrap();
        memory.write_u32(readfds + 4, listener).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, readfds, 0, 0, 0],
            ),
            1
        );
        assert_eq!(memory.read_u32(readfds).unwrap(), 1);
        assert_eq!(memory.read_u32(readfds + 4).unwrap(), listener);
        assert!(socket_read_ready(listener));

        memory.write_u32(remote_len, 16).unwrap();
        let accepted = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("accept"),
            &mut memory,
            &[listener, remote_addr, remote_len],
        );
        assert_ne!(accepted, INVALID_SOCKET);
        assert_eq!(memory.read_u32(remote_len).unwrap(), 16);
        drop(client);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("closesocket"),
                &mut memory,
                &[accepted],
            ),
            0
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("closesocket"),
                &mut memory,
                &[listener],
            ),
            0
        );
    }

    #[test]
    fn select_accept_loop_rearms_listener_after_each_client() {
        let _winsock_guard = locked_reset_for_tests();
        let probe = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let bind_addr = 0x3000_ca00;
        let readfds = 0x3000_cb00;
        let remote_addr = 0x3000_cc00;
        let remote_len = 0x3000_cd00;
        memory.write_sockaddr_v4(bind_addr, Ipv4Addr::LOCALHOST, port);
        let listener = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_STREAM, 0],
        );
        assert_ne!(listener, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("bind"),
                &mut memory,
                &[listener, bind_addr, 16],
            ),
            0
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("listen"),
                &mut memory,
                &[listener, 2],
            ),
            0
        );

        for _ in 0..4 {
            memory.write_fd_set(readfds, &[listener]);
            assert_eq!(
                dispatch_import(
                    &mut kernel,
                    1,
                    None,
                    Some("select"),
                    &mut memory,
                    &[0, readfds, 0, 0, 0],
                ),
                0
            );
            assert_eq!(memory.read_u32(readfds).unwrap(), 0);

            let client = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
            let client_port = client.local_addr().unwrap().port();
            let deadline = Instant::now() + Duration::from_secs(1);
            while !socket_read_ready(listener) && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(5));
            }
            assert!(socket_read_ready(listener));

            memory.write_fd_set(readfds, &[listener]);
            assert_eq!(
                dispatch_import(
                    &mut kernel,
                    1,
                    None,
                    Some("select"),
                    &mut memory,
                    &[0, readfds, 0, 0, 0],
                ),
                1
            );
            assert_eq!(memory.read_u32(readfds).unwrap(), 1);
            assert_eq!(memory.read_u32(readfds + 4).unwrap(), listener);

            memory.write_u32(remote_len, 16).unwrap();
            let accepted = dispatch_import(
                &mut kernel,
                1,
                None,
                Some("accept"),
                &mut memory,
                &[listener, remote_addr, remote_len],
            );
            assert_ne!(accepted, INVALID_SOCKET);
            assert_eq!(memory.read_u32(remote_len).unwrap(), 16);
            assert_eq!(memory.read_u16(remote_addr).unwrap(), AF_INET as u16);
            assert_eq!(
                u16::from_be_bytes([
                    memory.read_u8(remote_addr + 2).unwrap(),
                    memory.read_u8(remote_addr + 3).unwrap()
                ]),
                client_port
            );
            assert_eq!(
                [
                    memory.read_u8(remote_addr + 4).unwrap(),
                    memory.read_u8(remote_addr + 5).unwrap(),
                    memory.read_u8(remote_addr + 6).unwrap(),
                    memory.read_u8(remote_addr + 7).unwrap(),
                ],
                CE_GATEWAY_IP.octets()
            );
            assert!(!socket_read_ready(listener));

            assert_eq!(
                dispatch_import(
                    &mut kernel,
                    1,
                    None,
                    Some("closesocket"),
                    &mut memory,
                    &[accepted],
                ),
                0
            );
            drop(client);
        }

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("closesocket"),
                &mut memory,
                &[listener],
            ),
            0
        );
    }

    #[test]
    fn udp_select_readiness_waits_for_datagram_before_recvfrom() {
        let _winsock_guard = locked_reset_for_tests();
        let probe = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let bind_addr = 0x3000_9100;
        let readfds = 0x3000_9200;
        let recv_buf = 0x3000_9300;
        let remote_addr = 0x3000_9400;
        let remote_len = 0x3000_9500;
        memory.write_sockaddr_v4(bind_addr, Ipv4Addr::LOCALHOST, port);
        memory.write_u32(remote_len, 16).unwrap();
        let socket = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_DGRAM, 0],
        );
        assert_ne!(socket, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("bind"),
                &mut memory,
                &[socket, bind_addr, 16],
            ),
            0
        );
        assert!(socket_read_wait_candidate(socket));
        assert!(!socket_read_ready(socket));

        memory.write_u32(readfds, 1).unwrap();
        memory.write_u32(readfds + 4, socket).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, readfds, 0, 0, 0],
            ),
            0
        );
        assert_eq!(memory.read_u32(readfds).unwrap(), 0);

        let sender = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let sender_port = sender.local_addr().unwrap().port();
        sender
            .send_to(b"gram", (Ipv4Addr::LOCALHOST, port))
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        while !socket_read_ready(socket) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(socket_read_ready(socket));

        memory.write_u32(readfds, 1).unwrap();
        memory.write_u32(readfds + 4, socket).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, readfds, 0, 0, 0],
            ),
            1
        );
        assert_eq!(memory.read_u32(readfds).unwrap(), 1);
        assert_eq!(memory.read_u32(readfds + 4).unwrap(), socket);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("recvfrom"),
                &mut memory,
                &[socket, recv_buf, 4, 0, remote_addr, remote_len],
            ),
            4
        );
        assert_eq!(
            (0..4)
                .map(|offset| memory.read_u8(recv_buf + offset).unwrap())
                .collect::<Vec<_>>(),
            b"gram"
        );
        assert_eq!(memory.read_u32(remote_len).unwrap(), 16);
        assert_eq!(memory.read_u16(remote_addr).unwrap(), AF_INET as u16);
        assert_eq!(
            u16::from_be_bytes([
                memory.read_u8(remote_addr + 2).unwrap(),
                memory.read_u8(remote_addr + 3).unwrap()
            ]),
            sender_port
        );
        assert_eq!(
            [
                memory.read_u8(remote_addr + 4).unwrap(),
                memory.read_u8(remote_addr + 5).unwrap(),
                memory.read_u8(remote_addr + 6).unwrap(),
                memory.read_u8(remote_addr + 7).unwrap(),
            ],
            CE_GATEWAY_IP.octets()
        );
        assert!(!socket_read_ready(socket));
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
    }

    #[test]
    fn select_accepts_ce_nfds_values_and_preserves_ready_fd_set() {
        let _winsock_guard = locked_reset_for_tests();
        let probe = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let bind_addr = 0x3000_b100;
        let readfds = 0x3000_b200;
        memory.write_sockaddr_v4(bind_addr, Ipv4Addr::LOCALHOST, port);
        let socket = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_DGRAM, 0],
        );
        assert_ne!(socket, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("bind"),
                &mut memory,
                &[socket, bind_addr, 16],
            ),
            0
        );

        let sender = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        sender
            .send_to(b"nfds", (Ipv4Addr::LOCALHOST, port))
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        while !socket_read_ready(socket) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(socket_read_ready(socket));

        for nfds in [0, u32::MAX, 1] {
            memory.write_fd_set(readfds, &[socket]);
            assert_eq!(
                dispatch_import(
                    &mut kernel,
                    1,
                    None,
                    Some("select"),
                    &mut memory,
                    &[nfds, readfds, 0, 0, 0],
                ),
                1
            );
            assert_eq!(memory.read_u32(readfds).unwrap(), 1);
            assert_eq!(memory.read_u32(readfds + 4).unwrap(), socket);
            assert_eq!(kernel.threads.get_last_error(1), 0);
        }

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
    }

    #[test]
    fn select_rejects_invalid_fd_sets_and_socket_handles() {
        let _winsock_guard = locked_reset_for_tests();
        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let readfds = 0x3000_c100;

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, 0, 0, 0, 0],
            ),
            SOCKET_ERROR
        );
        assert_eq!(kernel.threads.get_last_error(1), WSAEINVAL);

        memory.write_u32(readfds, FD_SETSIZE + 1).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, readfds, 0, 0, 0],
            ),
            SOCKET_ERROR
        );
        assert_eq!(kernel.threads.get_last_error(1), WSAEINVAL);

        memory.write_fd_set(readfds, &[SOCKET_HANDLE_BASE + 0x1234]);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, readfds, 0, 0, 0],
            ),
            SOCKET_ERROR
        );
        assert_eq!(kernel.threads.get_last_error(1), WSAEBADF);
    }

    #[test]
    fn select_maps_fd_set_memory_faults_to_wsaefault() {
        let _winsock_guard = locked_reset_for_tests();
        let mut kernel = test_kernel();
        let readfds = 0x3000_c200;
        let mut read_fault_memory = FaultingTestMemory {
            read_faults: vec![readfds],
            ..FaultingTestMemory::default()
        };

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut read_fault_memory,
                &[0, readfds, 0, 0, 0],
            ),
            SOCKET_ERROR
        );
        assert_eq!(kernel.threads.get_last_error(1), WSAEFAULT);

        let socket = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut read_fault_memory,
            &[AF_INET, SOCK_DGRAM, 0],
        );
        assert_ne!(socket, INVALID_SOCKET);
        read_fault_memory.inner.write_fd_set(readfds, &[socket]);
        read_fault_memory.read_faults.clear();
        read_fault_memory.write_faults.push(readfds);

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut read_fault_memory,
                &[0, readfds, 0, 0, 0],
            ),
            SOCKET_ERROR
        );
        assert_eq!(kernel.threads.get_last_error(1), WSAEFAULT);

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("closesocket"),
                &mut read_fault_memory,
                &[socket],
            ),
            0
        );
    }

    #[test]
    fn select_filters_mixed_read_write_and_exception_fd_sets() {
        let _winsock_guard = locked_reset_for_tests();
        let udp_probe = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let udp_port = udp_probe.local_addr().unwrap().port();
        drop(udp_probe);
        let refused_probe = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let refused_port = refused_probe.local_addr().unwrap().port();
        drop(refused_probe);

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let udp_addr = 0x3000_c300;
        let refused_addr = 0x3000_c400;
        let readfds = 0x3000_c500;
        let writefds = 0x3000_c600;
        let exceptfds = 0x3000_c700;
        memory.write_sockaddr_v4(udp_addr, Ipv4Addr::LOCALHOST, udp_port);
        memory.write_sockaddr_v4(refused_addr, Ipv4Addr::LOCALHOST, refused_port);

        let udp = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_DGRAM, 0],
        );
        assert_ne!(udp, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("bind"),
                &mut memory,
                &[udp, udp_addr, 16],
            ),
            0
        );
        let sender = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        sender
            .send_to(b"mix", (Ipv4Addr::LOCALHOST, udp_port))
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        while !socket_read_ready(udp) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(socket_read_ready(udp));

        let failed = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_STREAM, 0],
        );
        assert_ne!(failed, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("connect"),
                &mut memory,
                &[failed, refused_addr, 16],
            ),
            SOCKET_ERROR
        );
        assert_eq!(kernel.threads.get_last_error(1), WSAECONNREFUSED);
        assert!(socket_except_ready(failed));

        memory.write_fd_set(readfds, &[udp]);
        memory.write_fd_set(writefds, &[udp]);
        memory.write_fd_set(exceptfds, &[failed]);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, readfds, writefds, exceptfds, 0],
            ),
            3
        );
        assert_eq!(memory.read_u32(readfds).unwrap(), 1);
        assert_eq!(memory.read_u32(readfds + 4).unwrap(), udp);
        assert_eq!(memory.read_u32(writefds).unwrap(), 1);
        assert_eq!(memory.read_u32(writefds + 4).unwrap(), udp);
        assert_eq!(memory.read_u32(exceptfds).unwrap(), 1);
        assert_eq!(memory.read_u32(exceptfds + 4).unwrap(), failed);
        assert_eq!(kernel.threads.get_last_error(1), 0);

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("closesocket"),
                &mut memory,
                &[failed],
            ),
            0
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("closesocket"),
                &mut memory,
                &[udp],
            ),
            0
        );
    }

    #[test]
    fn select_reports_tcp_peer_close_as_read_ready_for_zero_recv() {
        let _winsock_guard = locked_reset_for_tests();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let (accepted_tx, accepted_rx) = mpsc::channel();
        let (close_tx, close_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            accepted_tx.send(()).unwrap();
            close_rx.recv().unwrap();
            drop(stream);
        });

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let sockaddr = 0x3000_d100;
        let readfds = 0x3000_d200;
        let recv_buf = 0x3000_d300;
        memory.write_sockaddr_v4(sockaddr, Ipv4Addr::LOCALHOST, port);
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
        accepted_rx.recv().unwrap();
        assert!(!socket_read_ready(socket));

        close_tx.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        while !socket_read_ready(socket) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(socket_read_ready(socket));

        memory.write_fd_set(readfds, &[socket]);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, readfds, 0, 0, 0],
            ),
            1
        );
        assert_eq!(memory.read_u32(readfds).unwrap(), 1);
        assert_eq!(memory.read_u32(readfds + 4).unwrap(), socket);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("recv"),
                &mut memory,
                &[socket, recv_buf, 4, 0],
            ),
            0
        );
        assert_eq!(kernel.threads.get_last_error(1), 0);

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
    fn tcp_peer_write_shutdown_is_read_ready_but_keeps_guest_send_open() {
        let _winsock_guard = locked_reset_for_tests();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let (accepted_tx, accepted_rx) = mpsc::channel();
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        let (shutdown_done_tx, shutdown_done_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            accepted_tx.send(()).unwrap();
            shutdown_rx.recv().unwrap();
            stream.shutdown(Shutdown::Write).unwrap();
            shutdown_done_tx.send(()).unwrap();
            let mut input = [0; 4];
            stream.read_exact(&mut input).unwrap();
            assert_eq!(&input, b"pong");
        });

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let sockaddr = 0x3000_d600;
        let readfds = 0x3000_d700;
        let recv_buf = 0x3000_d800;
        let send_buf = 0x3000_d900;
        memory.write_sockaddr_v4(sockaddr, Ipv4Addr::LOCALHOST, port);
        memory.write_bytes_at(send_buf, b"pong");
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
        accepted_rx.recv().unwrap();
        assert!(!socket_read_ready(socket));

        shutdown_tx.send(()).unwrap();
        shutdown_done_rx.recv().unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        while !socket_read_ready(socket) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(socket_read_ready(socket));

        memory.write_fd_set(readfds, &[socket]);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, readfds, 0, 0, 0],
            ),
            1
        );
        assert_eq!(memory.read_u32(readfds).unwrap(), 1);
        assert_eq!(memory.read_u32(readfds + 4).unwrap(), socket);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("recv"),
                &mut memory,
                &[socket, recv_buf, 4, 0],
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
                Some("closesocket"),
                &mut memory,
                &[socket],
            ),
            0
        );
        server.join().unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn tcp_reset_is_read_ready_and_sets_so_error() {
        let _winsock_guard = locked_reset_for_tests();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let (accepted_tx, accepted_rx) = mpsc::channel();
        let (reset_tx, reset_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            accepted_tx.send(()).unwrap();
            reset_rx.recv().unwrap();
            force_tcp_reset_on_drop(&stream);
            drop(stream);
        });

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let sockaddr = 0x3000_da00;
        let readfds = 0x3000_db00;
        let recv_buf = 0x3000_dc00;
        let optval = 0x3000_dd00;
        let optlen = 0x3000_de00;
        memory.write_sockaddr_v4(sockaddr, Ipv4Addr::LOCALHOST, port);
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
        accepted_rx.recv().unwrap();
        assert!(!socket_read_ready(socket));

        reset_tx.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        while !socket_read_ready(socket) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(socket_read_ready(socket));

        memory.write_fd_set(readfds, &[socket]);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, readfds, 0, 0, 0],
            ),
            1
        );
        assert_eq!(memory.read_u32(readfds).unwrap(), 1);
        assert_eq!(memory.read_u32(readfds + 4).unwrap(), socket);

        memory.write_u32(optval, 0).unwrap();
        memory.write_u32(optlen, 4).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("getsockopt"),
                &mut memory,
                &[socket, SOL_SOCKET, SO_ERROR, optval, optlen],
            ),
            0
        );
        assert_eq!(memory.read_u32(optval).unwrap(), WSAECONNRESET);

        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("recv"),
                &mut memory,
                &[socket, recv_buf, 4, 0],
            ),
            SOCKET_ERROR
        );
        assert_eq!(kernel.threads.get_last_error(1), WSAECONNRESET);
        memory.write_u32(optval, 0).unwrap();
        memory.write_u32(optlen, 4).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("getsockopt"),
                &mut memory,
                &[socket, SOL_SOCKET, SO_ERROR, optval, optlen],
            ),
            0
        );
        assert_eq!(memory.read_u32(optval).unwrap(), WSAECONNRESET);
        memory.write_u32(optval, u32::MAX).unwrap();
        memory.write_u32(optlen, 4).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("getsockopt"),
                &mut memory,
                &[socket, SOL_SOCKET, SO_ERROR, optval, optlen],
            ),
            0
        );
        assert_eq!(memory.read_u32(optval).unwrap(), 0);

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
    fn select_recovers_after_zero_ready_poll_when_socket_becomes_readable() {
        let _winsock_guard = locked_reset_for_tests();
        let probe = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let bind_addr = 0x3000_d400;
        let readfds = 0x3000_d500;
        memory.write_sockaddr_v4(bind_addr, Ipv4Addr::LOCALHOST, port);
        let socket = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_DGRAM, 0],
        );
        assert_ne!(socket, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("bind"),
                &mut memory,
                &[socket, bind_addr, 16],
            ),
            0
        );

        memory.write_fd_set(readfds, &[socket]);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, readfds, 0, 0, 0],
            ),
            0
        );
        assert_eq!(memory.read_u32(readfds).unwrap(), 0);
        assert_eq!(kernel.threads.get_last_error(1), 0);

        let sender = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        sender
            .send_to(b"again", (Ipv4Addr::LOCALHOST, port))
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        while !socket_read_ready(socket) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(socket_read_ready(socket));

        memory.write_fd_set(readfds, &[socket]);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, readfds, 0, 0, 0],
            ),
            1
        );
        assert_eq!(memory.read_u32(readfds).unwrap(), 1);
        assert_eq!(memory.read_u32(readfds + 4).unwrap(), socket);

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
    }

    #[test]
    fn tcp_send_queues_peer_socket_wait_candidate() {
        let _winsock_guard = locked_reset_for_tests();
        let probe = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);

        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let bind_addr = 0x3000_6100;
        let readfds = 0x3000_6200;
        let send_buf = 0x3000_6300;
        let recv_buf = 0x3000_6400;
        memory.write_sockaddr_v4(bind_addr, Ipv4Addr::LOCALHOST, port);
        memory.write_bytes_at(send_buf, b"peer");
        let listener = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_STREAM, 0],
        );
        assert_ne!(listener, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("bind"),
                &mut memory,
                &[listener, bind_addr, 16],
            ),
            0
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("listen"),
                &mut memory,
                &[listener, 1],
            ),
            0
        );
        let client = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("socket"),
            &mut memory,
            &[AF_INET, SOCK_STREAM, 0],
        );
        assert_ne!(client, INVALID_SOCKET);
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("connect"),
                &mut memory,
                &[client, bind_addr, 16],
            ),
            0
        );
        memory.write_u32(readfds, 1).unwrap();
        memory.write_u32(readfds + 4, listener).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("select"),
                &mut memory,
                &[0, readfds, 0, 0, 0],
            ),
            1
        );
        let accepted = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("accept"),
            &mut memory,
            &[listener, 0, 0],
        );
        assert_ne!(accepted, INVALID_SOCKET);
        assert!(!socket_read_ready(client));

        let wait_id = kernel.register_blocked_waiter(
            71,
            0x771,
            vec![client],
            SchedulerBlockedWaitKind::Kernel,
            0,
            INFINITE,
        );
        assert_eq!(
            kernel.select_ready_blocked_waiter(1, 0, |blocked, _kernel| {
                blocked.wait_handles.iter().copied().any(socket_read_ready)
            }),
            None
        );
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("send"),
                &mut memory,
                &[accepted, send_buf, 4, 0],
            ),
            4
        );
        assert!(socket_read_ready(client));
        assert_eq!(
            kernel.select_ready_blocked_waiter(1, 0, |blocked, _kernel| {
                blocked.wait_handles.iter().copied().any(socket_read_ready)
            }),
            Some(wait_id)
        );
        kernel.remove_blocked_waiter(wait_id).unwrap();
        assert_eq!(
            dispatch_import(
                &mut kernel,
                1,
                None,
                Some("recv"),
                &mut memory,
                &[client, recv_buf, 4, 0],
            ),
            4
        );
        assert_eq!(
            (0..4)
                .map(|offset| memory.read_u8(recv_buf + offset).unwrap())
                .collect::<Vec<_>>(),
            b"peer"
        );
        for socket in [accepted, client, listener] {
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
        }
    }

    #[test]
    fn inet_and_byte_order_helpers_match_guest_layout() {
        let _winsock_guard = locked_reset_for_tests();
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

    #[test]
    fn isolated_nat_mode_reports_ce_gateway_and_guest_ip() {
        assert_eq!(
            network_mode(),
            WinsockNetworkMode::IsolatedNat {
                gateway: Ipv4Addr::new(10, 0, 0, 1),
                guest_ip: Ipv4Addr::new(10, 0, 0, 2),
            }
        );
    }

    #[test]
    fn gethostbyname_uses_isolated_ce_addresses_for_local_names() {
        let _winsock_guard = locked_reset_for_tests();
        let mut kernel = test_kernel();
        let mut memory = TestMemory::default();
        let name_ptr = 0x3000_2000;
        memory.write_bytes_at(name_ptr, b"fakece\0");

        let hostent = dispatch_import(
            &mut kernel,
            1,
            None,
            Some("gethostbyname"),
            &mut memory,
            &[name_ptr],
        );
        assert_ne!(hostent, 0);
        let addr_list = memory.read_u32(hostent + 12).unwrap();
        let addr_ptr = memory.read_u32(addr_list).unwrap();
        let addr = [
            memory.read_u8(addr_ptr).unwrap(),
            memory.read_u8(addr_ptr + 1).unwrap(),
            memory.read_u8(addr_ptr + 2).unwrap(),
            memory.read_u8(addr_ptr + 3).unwrap(),
        ];
        assert_eq!(addr, [10, 0, 0, 2]);
    }
}
