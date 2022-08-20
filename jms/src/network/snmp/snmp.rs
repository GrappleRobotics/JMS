use std::collections::HashMap;

use asn1_rs::{FromBer, oid, Oid};
use bytes::BufMut;
use futures::SinkExt;
use tokio::net::UdpSocket;
use tokio_stream::StreamExt;
use tokio_util::{codec::{Decoder, Encoder}, udp::UdpFramed};

use crate::{arena::{SharedArena, station::AllianceStationId}, models::Alliance};

use super::snmp_packet::{SNMPPacket, GenericTrap};

const OID_LINK_NAME: Oid = oid!(1.3.6.1.2.1.2.2.1.2);
const OID_VLAN_ID: Oid = oid!(1.3.6.1.4.1.9.9.68.1.2.2.1.2);

#[derive(Clone, Debug)]
pub struct VlanRequest {
  link: LinkStateNotification
}

#[derive(Clone, Debug)]
pub struct VlanResponse {
  link: LinkStateNotification,
  vlan: u64
}

#[derive(Clone, Debug)]
pub struct LinkStateNotification {
  id: u64,
  #[allow(dead_code)]
  name: String,
  up: bool
}

#[derive(Clone, Debug)]
pub enum SNMPRecvData {
  LinkStateNotif(LinkStateNotification),
  VlanStateNotif(VlanResponse)
}

pub struct SNMPCodec {
  vlan_matching_queue: HashMap<u64, VlanRequest>
}

impl SNMPCodec {
  pub fn new() -> Self {
    Self {
      vlan_matching_queue: HashMap::new()
    }
  }
}

impl SNMPCodec {
  fn process<'a>(&mut self, data: SNMPPacket<'a>) -> Result<Option<SNMPRecvData>, anyhow::Error> {
    match data {
      d if d.trap.is_some() => {
        let trap = d.trap.unwrap();
        match trap.generic {
          GenericTrap::LinkDown | GenericTrap::LinkUp => {
            let link_id_name = trap.binds.iter().find_map(|vb| {
              if vb.name.starts_with(&OID_LINK_NAME) && vb.string.is_some() {
                Some((vb.name.iter().unwrap().last().unwrap(), vb.string.clone().unwrap().0))
              } else {
                None
              }
            });

            if let Some(link_id_name) = link_id_name {
              Ok(Some(SNMPRecvData::LinkStateNotif(LinkStateNotification {
                id: link_id_name.0,
                name: link_id_name.1,
                up: trap.generic == GenericTrap::LinkUp
              })))
            } else {
              Ok(None)
            }
          },
          _ => Ok(None)
        }
      },
      d if d.get_response.is_some() => {
        let response = d.get_response.unwrap();
        let matched = self.vlan_matching_queue.remove(&response.request_id);
        match matched {
          Some(msg) => {
            Ok(response.binds.iter().find_map(|vb| {
              if vb.name.starts_with(&OID_VLAN_ID) && vb.number.is_some() {
                Some(SNMPRecvData::VlanStateNotif(VlanResponse {
                  link: msg.link.clone(),
                  vlan: vb.number.unwrap(),
                }))
              } else {
                None
              }
            }))
          },
          None => Ok(None),
        }
      }
      _ => Ok(None)
    }
  }
}

impl Decoder for SNMPCodec {
  type Item = SNMPRecvData;
  type Error = anyhow::Error;

  fn decode(&mut self, buf: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    if buf.is_empty() {
      Ok(None)
    } else {
      let (_, data) = SNMPPacket::from_ber(buf)?;
      let result = self.process(data);
      buf.clear();
      result
    }
  }
}

impl Encoder<VlanRequest> for SNMPCodec {
  type Error = anyhow::Error;

  fn encode(&mut self, item: VlanRequest, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
    // asn1_rs doesn't yet support serialisation of BER types, so this is a hacky, manually crafted packet instead.
    dst.put_slice(&[ 0x30, 0x30, 0x02, 0x01, 0x00, 0x04, 0x06, 0x70, 0x75, 0x62, 0x6c, 0x69, 0x63, 0xa0, 0x23 ]);
    
    // Request ID
    dst.put_slice(&[ 0x02, 0x04 ]);
    dst.put_u32(item.link.id as u32);

    dst.put_slice(&[0x02, 0x01, 0x00, 0x02, 0x01, 0x00]);

    // OID of Port
    let oid_arr = [1,3,6,1,4,1,9,9,68,1,2,2,1,2,item.link.id];
    let oid_buf = Oid::from(&oid_arr).unwrap();
    let oid_enc = oid_buf.as_bytes();

    dst.put_slice(&[0x30, (oid_enc.len() + 6) as u8, 0x30, (oid_enc.len() + 4) as u8, 0x06, oid_enc.len() as u8]);
    dst.put_slice(oid_enc);
    dst.put_slice(&[0x05, 0x00]);

    self.vlan_matching_queue.insert(item.link.id, item);

    Ok(())
  }
}

pub struct SNMPService {
  arena: SharedArena
}

impl SNMPService {
  pub fn new(arena: SharedArena) -> SNMPService {
    SNMPService {
      arena
    }
  }

  pub async fn run(self) -> anyhow::Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", 162)).await?;
    let mut framed = UdpFramed::new(socket, SNMPCodec::new());

    loop {
      match framed.next().await {
        Some(Ok(result)) => { 
          match result {
            (SNMPRecvData::LinkStateNotif(ls), addr) => {
              info!("Link state changed... asking for VLAN: {:?}", &ls);

              // Ask for VLAN ID. 5 times to make sure data gets through
              for _ in 0..5 {
                framed.send((VlanRequest { link: ls.clone() }, ( addr.ip(), 161 ).into())).await?;
              }
            },
            (SNMPRecvData::VlanStateNotif(vs), _) => {
              info!("Link state changed: {:?}", &vs);

              let station_id = match vs.vlan {
                10 => Some(AllianceStationId { alliance: Alliance::Blue, station: 1 }),
                20 => Some(AllianceStationId { alliance: Alliance::Blue, station: 2 }),
                30 => Some(AllianceStationId { alliance: Alliance::Blue, station: 3 }),
                40 => Some(AllianceStationId { alliance: Alliance::Red, station: 1 }),
                50 => Some(AllianceStationId { alliance: Alliance::Red, station: 2}),
                60 => Some(AllianceStationId { alliance: Alliance::Red, station: 3 }),
                _ => None
              };

              // Let the arena know if they're connected or not
              if let Some(station_id) = station_id {
                let mut arena = self.arena.lock().await;
                if let Some(station) = arena.station_mut(station_id) {
                  station.ds_eth = vs.link.up;
                }
              }
            },
          }
        }
        _ => ()
      }
    }
  }
}