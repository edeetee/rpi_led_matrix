use std::{net::{Ipv4Addr, UdpSocket, SocketAddr}, sync::mpsc::{sync_channel, TrySendError}};

#[derive(Debug, PartialEq, Clone)]
pub enum PdPacket {
    VoiceLevel(f32)
}

fn parse_packet(packet: &str) -> Option<PdPacket>{
    let mut splits = packet.split_ascii_whitespace();

    let path = splits.next()?;
    let number = splits.next()?.strip_suffix(";")?.parse().ok()?;

    match path {
        _ if path.starts_with("/voicelevel") => {
            Some(PdPacket::VoiceLevel(number))
        },
        _ => None
    }
}

pub fn receive() -> std::sync::mpsc::Receiver<PdPacket> {
    let addr: SocketAddr = (Ipv4Addr::UNSPECIFIED, 2000).into();
    let socket = UdpSocket::bind(addr).unwrap();

    let mut buf = [0u8; 128];

    let (pd_tx, pd_rx) = sync_channel(0);

    std::thread::spawn(move || {
        loop {
            let read_bytes = socket.recv(&mut buf).unwrap();

            let str = std::str::from_utf8(buf[..read_bytes].into()).unwrap();
                let data = parse_packet(&str);

                if let Some(data) = &data {
                    // println!("try_send:\t{data:?}");
                    let resp = pd_tx.try_send(data.clone());

                    if let Err(TrySendError::Disconnected(_)) = resp {
                        panic!("Receiver has been disconnected!");
                    }
                }
        }
    });
    
    pd_rx
}


#[cfg(test)]
mod tests {
    use super::{parse_packet, PdPacket};

    #[test]
    fn voicelevel() {
        assert_eq!(parse_packet("/voicelevel/ 0.31;"), Some(PdPacket::VoiceLevel(0.31)));
        assert_eq!(parse_packet("/voicelevel/ 0.0;"), Some(PdPacket::VoiceLevel(0.0)));
        assert_eq!(parse_packet("/voicelevel 10.0;"), Some(PdPacket::VoiceLevel(10.0)));
    }
    
    #[test]
    fn none(){
        assert_eq!(parse_packet("/voicelevel/;"), None);
        assert_eq!(parse_packet("/voicel"), None);
    }
}
