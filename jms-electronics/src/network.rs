use binmarshal::{BitView, BitWriter, Demarshal, VecBitWriter};
use bounded_static::ToBoundedStatic;
use bytes::Buf;
use grapple_frc_msgs::{grapple::{write_direct, MaybeFragment, TaggedGrappleMessage}, ManufacturerMessage, MessageId};
use log::error;
use pnet::{datalink::{self, Channel as DatalinkChannel, DataLinkReceiver, DataLinkSender, NetworkInterface}, packet::{ethernet::{EtherType, EthernetPacket, MutableEthernetPacket}, MutablePacket, Packet}, util::MacAddr};
use tokio::sync::mpsc;
use tokio_util::codec::{Decoder, Encoder};

type Channel = (tokio::sync::mpsc::Sender<EthernetPacket<'static>>, tokio::sync::mpsc::Receiver<EthernetPacket<'static>>);

pub struct JMSElectronicsL2Framed {
  channel: Channel,
  our_mac: MacAddr
}

impl JMSElectronicsL2Framed {
  pub fn new(iface: NetworkInterface) -> Self {
    let (send_tx, mut send_rx) = mpsc::channel::<EthernetPacket<'static>>(16);
    let (recv_tx, recv_rx) = mpsc::channel(16);
    
    let (mut tx, rx) = match datalink::channel(&iface, Default::default()) {
      Ok(DatalinkChannel::Ethernet(tx, rx)) => (tx, rx),
      Ok(_) => panic!("Unhandled channel type"),
      Err(e) => panic!("Could not create datalink channel: {}", e)
    };

    // Send Thread
    std::thread::spawn(move || {
      loop {
        match send_rx.blocking_recv() {
          Some(pkt) => {
            tx.build_and_send(1, pkt.packet().len(),
                &mut |mut new_packet| {
                let mut new_packet = MutableEthernetPacket::new(new_packet).unwrap();

                new_packet.clone_from(&pkt);

                new_packet.set_source(pkt.get_source());
                new_packet.set_destination(pkt.get_destination());
                new_packet.set_ethertype(EtherType(0x7171));
            });
          },
          None => {
            error!("Send Pipe Broken");
            break
          },
        }
      }
    });
    
    // Recv Thread
    std::thread::spawn(move || {
      let mut rx = rx;
      loop {
        match rx.next() {
          Ok(pkt) => {
            let packet = EthernetPacket::owned(pkt.to_vec()).unwrap();
            if packet.get_ethertype().0 == 0x7172 {
              recv_tx.blocking_send(packet).ok();
            }
          },
          Err(e) => {
            error!("Socket Error: {}", e);
            break
          }
        }
      }
    });

    return Self { channel: (send_tx, recv_rx), our_mac: iface.mac.unwrap() }
  }

  pub async fn next(&mut self) -> Option<EthernetPacket<'static>> {
    self.channel.1.recv().await
  }

  pub async fn next_framed(&mut self) -> Option<(MacAddr, TaggedGrappleMessage<'static>)> {
    let pkt = self.channel.1.recv().await?;

    if pkt.packet().len() > 0 {
      let mut view = BitView::new(&pkt.payload());

      let result = match view.take::<4>(4, 0) {
        Ok(arr) => {
          let id: MessageId = u32::from_le_bytes(*arr.0).into();
          match ManufacturerMessage::read(&mut view, id.clone()) {
            Ok(ManufacturerMessage::Grapple(MaybeFragment::Message(msg))) => {
              Ok(Some(TaggedGrappleMessage::new(id.device_id, msg.to_static())))
            },
            _ => Ok(None),
          }
        },
        Err(_) => Err(anyhow::anyhow!("Demarshal Error Error"))
      };

      return result.ok().and_then(|x| x.map(|y| (pkt.get_source(), y)))
    }
    return None
  }

  pub async fn send(&mut self, packet: EthernetPacket<'static>) -> Result<(), mpsc::error::SendError<EthernetPacket<'static>>> {
    self.channel.0.send(packet).await
  }

  pub async fn send_framed<'a>(&mut self, addr: MacAddr, msg: TaggedGrappleMessage<'a>) -> anyhow::Result<()> {
    let mut writer = VecBitWriter::new();
    write_direct(&mut writer, msg).map_err(|_| anyhow::anyhow!("Writer Error!"))?;
    let serialised = writer.slice();

    let v = vec![0x00; 14 + serialised.len()];
    let mut packet = MutableEthernetPacket::owned(v).ok_or(anyhow::anyhow!("Could not construct packet (mutable)"))?;

    packet.payload_mut().clone_from_slice(serialised);
    packet.set_source(self.our_mac);
    packet.set_destination(addr);

    let packet = EthernetPacket::owned(packet.packet().to_vec()).ok_or(anyhow::anyhow!("Could not construct packet (immutable)"))?;

    self.send(packet).await?;

    Ok(())
  }
}

pub struct JMSElectronicsCodec { }

impl Decoder for JMSElectronicsCodec {
  type Item = TaggedGrappleMessage<'static>;
  type Error = anyhow::Error;

  fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    if src.len() < 4 { return Ok(None) }

    let mut view = BitView::new(&src[..]);

    let result = match view.take::<4>(4, 0) {
      Ok(arr) => {
        let id: MessageId = u32::from_le_bytes(*arr.0).into();
        match ManufacturerMessage::read(&mut view, id.clone()) {
          Ok(ManufacturerMessage::Grapple(MaybeFragment::Message(msg))) => {
            Ok(Some(TaggedGrappleMessage::new(id.device_id, msg.to_static())))
          },
          _ => Ok(None),
        }
      },
      Err(_) => Err(anyhow::anyhow!("Demarshal Error Error"))
    };

    src.advance(src.len());

    result
  }
}

impl<'a> Encoder<TaggedGrappleMessage<'a>> for JMSElectronicsCodec {
  type Error = anyhow::Error;

  fn encode(&mut self, item: TaggedGrappleMessage<'a>, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
    let mut writer = VecBitWriter::new();
    write_direct(&mut writer, item).map_err(|_| anyhow::anyhow!("Writer Error!"))?;
    dst.extend_from_slice(writer.slice());
    Ok(())
  }
}