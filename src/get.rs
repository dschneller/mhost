use defaults::{DEFAULT_DNS_SERVERS, DEFAULT_RECORD_TYPES};
use dns::{Source, Server};
use ungefiltert_surfen::{self, Server as UngefiltertServer};

use futures::{Future, future};
use resolv_conf;
use std::fs::File;
use std::io::Read;
use std::net::IpAddr;
use std::str::FromStr;
use tokio_core::reactor::Handle;
use trust_dns::rr::RecordType;


pub fn dns_servers(
    loop_handle: &Handle,
    servers: Option<Vec<String>>,
    use_predefined_server: bool,
    dont_use_local_servers: bool,
    ungefiltert_surfen_ids: Option<Vec<String>>,
) -> Box<Future<Item=Vec<Server>, Error=Error>> {
    let from_args = future::ok::<Vec<Server>, Error>({
        let mut dns_servers: Vec<Server> = Vec::new();
        if let Some(servers) = servers {
            dns_servers.extend(servers.into_iter()
                .map(|server| {
                    let ip_addr = IpAddr::from_str(&server).unwrap();
                    Server::udp_from_with_port(ip_addr, 53u16, Source::Additional)
                }));
        }
        if use_predefined_server {
            dns_servers.extend(DEFAULT_DNS_SERVERS.iter()
                .map(|server| {
                    let ip_addr = IpAddr::from_str(server).unwrap();
                    Server::udp_from_with_port(ip_addr, 53u16, Source::Predefined)
                }));
        }
        if !dont_use_local_servers {
            dns_servers.extend(dns_servers_from_resolv_conf().unwrap().into_iter()
                .map(|ip_addr| {
                    Server::udp_from_with_port(ip_addr, 53u16, Source::Local)
                })
            );
        }

        dns_servers
    }).map_err(move |e| {
        Error::with_chain(e, ErrorKind::ServerIpAddrParsingError)
    });

    let us: Vec<_> = ungefiltert_surfen_ids
        .unwrap_or_else(|| vec![])
        .iter()
        .map(|id| ungefiltert_surfen::retrieve_servers(loop_handle, id))
        .collect();
    let from_ungefiltert = future::join_all(us)
        .map(move |answers| {
            answers
                .into_iter()
                .fold(Vec::new(), |mut acc, servers: Vec<UngefiltertServer>| {
                    acc.extend(servers);
                    acc
                })
                .iter()
                .map(|server| {
                    let ip_addr = IpAddr::from_str(&server.ip).unwrap();
                    Server::udp_from_with_port(ip_addr, 53u16, Source::Ungefiltert)
                })
                .collect()
        })
        .map_err(move |e| e.into());

    Box::new(from_args.join(from_ungefiltert).map(
        |(mut r1, r2): (Vec<_>, Vec<_>)| {
            r1.extend(r2);
            r1
        },
    ))
}

pub fn resolv_conf() -> Result<resolv_conf::Config> {
    let mut buf = Vec::with_capacity(4096);
    let mut f = File::open("/etc/resolv.conf").chain_err(
        || ErrorKind::ResolvConfError,
    )?;
    f.read_to_end(&mut buf).unwrap();
    let cfg = resolv_conf::Config::parse(&buf[..]).chain_err(|| {
        ErrorKind::ResolvConfError
    })?;

    Ok(cfg)
}

fn dns_servers_from_resolv_conf() -> Result<Vec<IpAddr>> {
    let cfg = resolv_conf()?;
    Ok(cfg.nameservers)
}

pub fn record_types(record_types: Option<Vec<String>>) -> Result<Vec<RecordType>> {
    let record_types = if let Some(rt) = record_types {
        rt.iter()
            .map(|rt| RecordType::from_str(&rt.to_uppercase()).unwrap())
            .collect()
    } else {
        DEFAULT_RECORD_TYPES
            .iter()
            .map(|rt| RecordType::from_str(&rt.to_uppercase()).unwrap())
            .collect()
    };

    Ok(record_types)
}

error_chain! {
    errors {
        ResolvConfError {
            description("failed to parse /etc/resolv.conf")
            display("failed to parse /etc/resolv.cons")
        }

        ServerIpAddrParsingError {
            description("failed to parse server IP address")
            display("failed to parse server IP address")
        }

        ResoureRecordTypeParsingError {
            description("failed to parse resource record type")
            display("failed to parse resource record type")
        }
    }

    links {
        Ungefiltert(ungefiltert_surfen::Error, ungefiltert_surfen::ErrorKind);
    }
}
