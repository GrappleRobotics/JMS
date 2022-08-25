use std::net::Ipv4Addr;

use asn1_rs::{BerSequence, Tagged, Oid, Null, Any, ApplicationImplicit, SequenceOf};

#[derive(Debug, BerSequence)]
pub struct SNMPPacket<'a> {
  pub version: u8,
  pub community: OctetString,
  // data: 
  #[tag_implicit(0)]
  #[optional]
  pub get: Option<SNMPPDU<'a>>,

  #[tag_implicit(1)]
  #[optional]
  pub get_next: Option<SNMPPDU<'a>>,

  #[tag_implicit(2)]
  #[optional]
  pub get_response: Option<SNMPPDU<'a>>,

  #[tag_implicit(3)]
  #[optional]
  pub set: Option<SNMPPDU<'a>>,

  #[tag_implicit(4)]
  #[optional]
  pub trap: Option<SNMPTrap<'a>>,
}

pub type RequestID = u64;
pub type ErrorStatus = u64;
pub type ErrorIndex = u64;

#[derive(Debug, Clone)]
pub struct IpAddress(pub Ipv4Addr);

impl<'a> TryFrom<Any<'a>> for IpAddress {
  type Error = asn1_rs::Error;

  fn try_from(value: Any<'a>) -> Result<Self, Self::Error> {
    let u = ApplicationImplicit::<&'a [u8], Self::Error, 0>::try_from(value)?.into_inner();
    Ok(IpAddress(Ipv4Addr::new(u[0], u[1], u[2], u[3])))
  }
}

#[derive(Debug, Clone)]
pub struct OctetString(pub String);

impl<'a> TryFrom<Any<'a>> for OctetString {
  type Error = asn1_rs::Error;

  fn try_from(value: Any<'a>) -> Result<Self, Self::Error> {
    match std::str::from_utf8(&value.as_bytes()) {
      Ok(s) => Ok(Self(s.to_owned())),
      Err(_) => Err(asn1_rs::Error::StringInvalidCharset),
    }
  }
}

pub type Counter = ApplicationImplicit<u32, asn1_rs::Error, 1>;
pub type Gauge = ApplicationImplicit<u32, asn1_rs::Error, 2>;
pub type TimeTicks<'a> = ApplicationImplicit<u32, asn1_rs::Error, 3>;
pub type Opaque<'a> = ApplicationImplicit<&'a [u8], asn1_rs::Error, 4>;

#[derive(Debug, BerSequence)]
pub struct VarBind<'a> {
  pub name: Oid<'a>,
  #[optional]
  pub number: Option<u64>,
  #[optional]
  pub string: Option<OctetString>,
  #[optional]
  pub object: Option<Oid<'a>>,
  #[optional]
  pub null: Option<Null>,
  #[optional]
  pub ip: Option<IpAddress>,
  #[optional]
  pub counter: Option<Counter>,
  #[optional]
  pub gauge: Option<Gauge>,
  #[optional]
  pub time: Option<TimeTicks<'a>>,
  #[optional]
  pub opaque: Option<Opaque<'a>>,
}

pub type VarBindList<'a> = SequenceOf<VarBind<'a>>;

#[derive(Debug, BerSequence)]
pub struct SNMPPDU<'a> {
  pub request_id: RequestID,
  pub error_status: ErrorStatus,
  pub error_index: ErrorIndex,
  pub binds: VarBindList<'a>
}

#[derive(Debug, strum_macros::FromRepr, PartialEq)]
#[repr(u8)]
pub enum GenericTrap {
  ColdStart = 0,
  WarmStart = 1,
  LinkDown = 2,
  LinkUp = 3,
  AuthFailure = 4,
  EgpNeighbourLoss = 5,
  Specific = 6
}

impl<'a> TryFrom<Any<'a>> for GenericTrap {
  type Error = asn1_rs::Error;

  fn try_from(value: Any<'a>) -> Result<Self, Self::Error> {
    match GenericTrap::from_repr(value.as_u8()?) {
      Some(v) => Ok(v),
      None => Err(asn1_rs::Error::InvalidValue { tag: value.tag(), msg: "Not a valid generic trap".to_owned() }),
    }
  }
}

#[derive(Debug, BerSequence)]
pub struct SNMPTrap<'a> {
  pub enterprise: Oid<'a>,
  pub addr: IpAddress,
  pub generic: GenericTrap,
  pub specific: u32,
  pub time: TimeTicks<'a>,
  pub binds: VarBindList<'a>
}